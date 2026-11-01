use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) fn original(family: &str) -> &'static str {
    match family {
        "queen-bee" => {
            include_str!("../../../../game_resources/authoring/queen-bee/fragment.skill.yaml")
        }
        "rudolta" => {
            include_str!("../../../../game_resources/authoring/rudolta/fragment.skill.yaml")
        }
        _ => panic!("unknown canonical family"),
    }
}
pub(crate) fn catalog_original(family: &str) -> &'static str {
    match family {
        "queen-bee" => {
            include_str!("../../../../game_resources/authoring/queen-bee/fragment.catalog.json")
        }
        "rudolta" => {
            include_str!("../../../../game_resources/authoring/rudolta/fragment.catalog.json")
        }
        _ => panic!("unknown canonical family"),
    }
}
fn inputs() -> Json {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_deferred_effects.values.json"
    ))
    .unwrap()
}
pub(crate) fn resolved(family: &str) -> Json {
    let mut document = parse_yaml(original(family)).unwrap();
    replace_holes(&mut document, &inputs()[family]);
    document
}
pub(crate) fn catalog(
    family: &str,
) -> crate::game::data::skill_fragment_data::SkillFragmentMetadata {
    let mut document: Json = serde_json::from_str(catalog_original(family)).unwrap();
    replace_holes(&mut document, &inputs()[family]);
    serde_json::from_value(document).unwrap()
}
pub(crate) fn edit_power(document: &mut Json, family: &str) {
    if family == "rudolta" {
        document["values"][0]["value"]["Amount"]["root"]["Ratio"]["numerator"]["Mul"]["scalar"]
            ["Const"]["value"] = json!(200);
    } else {
        document["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["State"]
            ["program"]["handlers"][0]["body"][1]["HealOwner"]["amount"]["root"]["Ratio"]
            ["numerator"]["Mul"]["scalar"]["Const"]["value"] = json!(10);
    }
}
#[test]
fn canonical_deferred_sources_preserve_unresolved_leaves_and_independent_lifetimes() {
    for family in ["queen-bee", "rudolta"] {
        assert_eq!(
            compile_yaml(original(family)).unwrap_err().code,
            DiagnosticCode::Unresolved
        );
        let original = parse_yaml(original(family)).unwrap();
        let document = resolved(family);
        let mut pending = vec![];
        holes(&original, "", &mut pending);
        assert_eq!(pending.len(), if family == "queen-bee" { 7 } else { 8 });
        for (path, marker) in pending {
            let mut one = document.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let diagnostic = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(
                diagnostic.code,
                DiagnosticCode::Unresolved,
                "{family} {path}: {diagnostic:?}"
            );
            assert!(!diagnostic.source.definition.is_empty());
        }
        let artifact = compile_json(&document.to_string()).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let native = restored.load_native().unwrap();
        let skill = &native.skills[0];
        assert_eq!(skill.steps.len(), 1);
        assert_eq!(artifact.source_digest, restored.source_digest);
        let metadata = catalog(family);
        let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = metadata.effect
        else {
            panic!()
        };
        assert_eq!(forms["base"].skill_id, skill.id);
        assert_eq!(forms["base"].resonance_max, 5);
        assert!(serde_json::from_str::<
            crate::game::data::skill_fragment_data::SkillFragmentMetadata,
        >(catalog_original(family))
        .is_err());
        if family == "queen-bee" {
            assert_eq!(metadata.compatibility.allowed_weapon_archetypes.len(), 1);
            let crate::game::ability::SkillEffectDef::Sustained {
                definition:
                    crate::game::ability::sustained::SustainedEffectDef::State { program, .. },
            } = &skill.steps[0].effects[0].op
            else {
                panic!()
            };
            assert!(!program.owns_use && !program.end_on_source_exit);
        } else {
            let crate::game::ability::DeliveryDef::TileArea { area, .. } = &skill.steps[0].delivery
            else {
                panic!()
            };
            assert_eq!(area.duration_ms, 201);
            assert_eq!(area.tick_interval_ms, Some(100));
            assert_eq!(area.emission.as_ref().unwrap().delay_ms, 200);
            assert!(area.emission.as_ref().unwrap().include_air);
            let mut invalid = document.clone();
            invalid["values"][1]["value"]["Delivery"]["TileArea"]["area"]["duration_ms"] =
                json!(202);
            assert!(compile_json(&invalid.to_string()).is_err());
        }
    }
}
