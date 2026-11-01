use super::*;
use serde_json::{json, Value as Json};

pub(crate) const ORIGINAL: &str =
    include_str!("../../../../game_resources/authoring/little-red/fragment.skill.yaml");
const CATALOG: &str =
    include_str!("../../../../game_resources/authoring/little-red/fragment.catalog.json");
fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_input_plan.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved() -> Json {
    let mut d = parse_yaml(ORIGINAL).unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &inputs());
    d
}
pub(crate) fn program(d: &mut Json) -> &mut Json {
    &mut d["values"][0]["value"]["Effects"][0]["op"]["InputPlan"]["program"]
}
pub(crate) fn catalog() -> crate::game::data::skill_fragment_data::SkillFragmentMetadata {
    let mut d: Json = serde_json::from_str(CATALOG).unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &inputs());
    serde_json::from_value(d).unwrap()
}
pub(crate) fn relics() -> crate::game::data::relic_data::RelicDatabase {
    let mut d: Json = serde_json::from_str(include_str!(
        "../../../../game_resources/authoring/relics/input-plan-modifiers.catalog.json"
    ))
    .unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &inputs());
    let additional: crate::game::data::relic_data::RelicDatabase =
        serde_json::from_value(d).unwrap();
    let mut shared = super::canonical_knight_tests::relics();
    shared.relics.extend(additional.relics);
    shared
}

#[test]
fn canonical_input_plan_source_preserves_known_policy_and_rejects_each_unknown() {
    let original = parse_yaml(ORIGINAL).unwrap();
    let document = resolved();
    let mut holes = vec![];
    super::canonical_area_tests::holes(&original, "", &mut holes);
    assert_eq!(holes.len(), 42);
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
    // The old graph used a technical Concept slug. Match its explicitly declared
    // family to the preserved catalog origin before comparing runtime semantics.
    let mut legacy = crate::authoring::node_graph::tests::input_plan_fixture();
    let family = legacy["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["kind"] == "skill-hostile-family-weakness")
        .unwrap();
    family["values"]["family"] = json!("f-01-57_little_red");
    let old = crate::authoring::node_graph::compile(
        &legacy.to_string(),
        "fragment_little_red_volley".into(),
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
    for skill in [&native.skills[0], &old.skills[0]] {
        let crate::game::ability::SkillUseLifetime::ActionAndEffects { effects } =
            &skill.use_lifetime
        else {
            panic!()
        };
        assert_eq!(effects.len(), 1);
        assert!(skill
            .use_lifetime
            .owns_effect(&skill.steps[0].id, &skill.steps[0].effects[0].id));
    }
    let mut document = document;
    let p = program(&mut document);
    assert_eq!(
        p["cohort"]["weakness_percent"],
        json!({"numerator":50,"denominator":1})
    );
    assert_eq!(p["cohort"]["kill_gain"], 1);
    assert_eq!(p["enhanced_projectile"]["after"][0]["duration_ms"], 2000);
    assert_eq!(p["allow_duplicates"], false);
    let relics = relics();
    let ids: std::collections::BTreeSet<_> = relics.relics.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids.len(), relics.relics.len());
    for m in p["modifiers"].as_array().unwrap() {
        assert!(ids.contains(m["relic_definition"].as_str().unwrap()));
        for required in m["requires"].as_array().unwrap() {
            assert!(ids.contains(required.as_str().unwrap()));
        }
    }
    let metadata = catalog();
    let crate::game::data::skill_fragment_data::SkillFragmentOrigin::Abnormality { abnormality_id } =
        metadata.origin.as_ref().unwrap()
    else {
        panic!()
    };
    assert_eq!(p["cohort"]["weakness_family"], json!(abnormality_id));
    assert_eq!(metadata.id.as_str(), "fragment_little_red_trace");
    assert_eq!(
        metadata.uuid.to_string(),
        "730e8400-e29b-41d4-a716-446655440018"
    );
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = metadata.effect
    else {
        panic!()
    };
    assert_eq!(forms.len(), 1);
    assert_eq!(forms["base"].skill_id, native.skills[0].id);
    assert_eq!(forms["base"].resonance_max, 100);
    assert!(
        serde_json::from_str::<crate::game::data::skill_fragment_data::SkillFragmentMetadata>(
            CATALOG
        )
        .is_err()
    );
}

#[test]
fn canonical_input_plan_rejects_conflicting_admission_and_release_contracts() {
    let document = resolved();
    for fault in ["admission", "prepare", "completion", "stock", "weakness"] {
        let mut d = document.clone();
        match fault {
            "admission" => d["skills"][0]["admission"]["capacity"] = json!(99),
            "prepare" => d["skills"][0]["prepare_ms"] = json!(1),
            "completion" => d["skills"][0]["body"]["Action"]["wait"] = json!("EffectsSettled"),
            "stock" => program(&mut d)["normal"]["capacity"] = json!(0),
            _ => {
                program(&mut d)["cohort"]["weakness_percent"] =
                    json!({"numerator":3,"denominator":2})
            }
        }
        assert_eq!(
            compile_json(&d.to_string()).unwrap_err().code,
            DiagnosticCode::Native,
            "{fault}"
        );
    }
}
