use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) fn original(bloom: bool) -> &'static str {
    if bloom {
        include_str!("../../../../game_resources/authoring/knight-of-despair/bloom.skill.yaml")
    } else {
        include_str!("../../../../game_resources/authoring/knight-of-despair/base.skill.yaml")
    }
}
pub(crate) const CATALOG: &str =
    include_str!("../../../../game_resources/authoring/knight-of-despair/fragment.catalog.json");
fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_knight.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved(bloom: bool) -> Json {
    let mut document = parse_yaml(original(bloom)).unwrap();
    replace_holes(&mut document, &inputs());
    document
}
pub(crate) fn catalog() -> crate::game::data::skill_fragment_data::SkillFragmentMetadata {
    let mut document: Json = serde_json::from_str(CATALOG).unwrap();
    replace_holes(&mut document, &inputs());
    serde_json::from_value(document).unwrap()
}
pub(crate) fn relics() -> crate::game::data::relic_data::RelicDatabase {
    let mut document: Json = serde_json::from_str(include_str!(
        "../../../../game_resources/authoring/relics/relation-modifiers.catalog.json"
    ))
    .unwrap();
    replace_holes(&mut document, &inputs());
    let additional: crate::game::data::relic_data::RelicDatabase =
        serde_json::from_value(document).unwrap();
    let mut shared = super::canonical_area_tests::relics();
    shared.relics.extend(additional.relics);
    shared
}
pub(crate) fn program(document: &mut Json) -> &mut Json {
    &mut document["values"][0]["value"]["Effects"][0]["op"]["RelationAction"]["program"]
}

#[test]
fn canonical_knight_sources_preserve_known_rules_and_reject_every_unresolved_leaf() {
    let metadata = catalog();
    assert!(
        serde_json::from_str::<crate::game::data::skill_fragment_data::SkillFragmentMetadata>(
            CATALOG
        )
        .is_err()
    );
    assert_eq!(
        metadata.uuid.to_string(),
        "730e8400-e29b-41d4-a716-446655440010"
    );
    assert_eq!(metadata.compatibility.block_capacity_min, Some(1));
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = metadata.effect
    else {
        panic!()
    };
    let relics = relics();
    let ids: std::collections::BTreeSet<_> = relics.relics.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids.len(), relics.relics.len());
    for bloom in [false, true] {
        let original = parse_yaml(original(bloom)).unwrap();
        let document = resolved(bloom);
        let mut pending = vec![];
        holes(&original, "", &mut pending);
        assert_eq!(pending.len(), if bloom { 21 } else { 22 });
        for (path, marker) in pending {
            let mut one = document.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let diagnostic = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(
                diagnostic.code,
                DiagnosticCode::Unresolved,
                "{path}: {diagnostic:?}"
            );
            assert!(!diagnostic.source.definition.is_empty());
        }
        let artifact = compile_json(&document.to_string()).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let native = restored.load_native().unwrap();
        let skill = &native.skills[0];
        let old = crate::authoring::node_graph::compile(
            &crate::authoring::node_graph::tests::relation_action_fixture(bloom).to_string(),
            "comparison".into(),
        )
        .unwrap()
        .load_native()
        .unwrap();
        // Identity/name differ, but the independently reviewed legacy fixture
        // is an exact reference for this explicit technical completion.
        assert_eq!(
            serde_json::to_value(&skill.steps[0].effects).unwrap(),
            serde_json::to_value(&old.skills[0].steps[0].effects).unwrap()
        );
        assert_eq!(skill.cast_targeting, old.skills[0].cast_targeting);
        assert_eq!(skill.resonance_admission, old.skills[0].resonance_admission);
        assert!(skill
            .use_lifetime
            .owns_effect(&skill.steps[0].id, &skill.steps[0].effects[0].id));
        let form = if bloom { "bloom" } else { "base" };
        assert_eq!(forms[form].skill_id, skill.id);
        assert_eq!(forms[form].resonance_max, 100);
        let mut known = document.clone();
        let p = program(&mut known);
        assert_eq!(p["stock_capacity"], 4);
        assert_eq!(p["periodic_gain"], 1);
        assert_eq!(p["shield_threshold"], 10);
        assert_eq!(p["shield_duration_ms"], 15000);
        assert_eq!(p["shield_percent"], json!({"numerator":50,"denominator":1}));
        assert_eq!(
            p["mode_damage_percent"],
            json!({"numerator":5,"denominator":1})
        );
        for modifier in p["modifiers"].as_array().unwrap() {
            assert!(ids.contains(modifier["relic_definition"].as_str().unwrap()));
        }
        if bloom {
            assert_eq!(p["plans"][8]["op"]["Timed"]["interval_ms"], 500);
            assert_eq!(
                p["active"]["self_damage"],
                json!([1000,{"numerator":10,"denominator":1},"Physical"])
            );
            assert_eq!(p["active"]["manual_stop"], true);
            assert_eq!(p["active"]["interrupt_on_silence"], true);
            assert_eq!(
                p["plans"][1]["op"]["Strike"]["after"][0]["duration_ms"],
                2000
            );
        } else {
            assert_eq!(p["normal"]["next_basic"], true);
            assert_eq!(
                p["plans"][10]["op"]["Strike"]["range"]["pattern"],
                json!({"rows":["X","X","@"],"include_anchor_tile":false})
            );
            assert_eq!(p["plans"][23]["op"]["Repeat"]["timing"], "FixedTotal");
            assert_eq!(
                p["plans"][22]["op"]["Strike"]["ramp"],
                json!({"numerator":10,"denominator":1})
            );
        }
    }
}
