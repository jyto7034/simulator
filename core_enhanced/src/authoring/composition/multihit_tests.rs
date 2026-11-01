use super::*;
use serde_json::{json, Value as Json};

fn document() -> Json {
    let mut doc = parse_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/woodsman_monk.skill.yaml"
    ))
    .unwrap();
    doc["skills"][0]["modifiers"] = json!([{
        "id":"same-cast", "relic_definition":"deep-wedge", "variant":"Normal",
        "change":{"Multihit":{"increment_percent":{"Literal":{"Count":2}},"cap_percent":{"Literal":{"Count":20}}}}
    }]);
    doc
}

#[test]
fn composition_multihit_expands_one_counter_across_calls_with_exact_origins() {
    let mut doc = document();
    doc["values"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"step-rate","value":{"Count":3}}));
    doc["skills"][0]["modifiers"][0]["change"]["Multihit"]["increment_percent"] =
        json!({"Shared":"step-rate"});
    let artifact = compile_json(&doc.to_string()).unwrap();
    let native = Artifact::from_ron(&artifact.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let policy = native.skills[0].action_policy.as_ref().unwrap();
    assert_eq!(policy.modifiers.len(), 1);
    let crate::game::ability::action::ActionChange::Multihit {
        steps,
        increment_percent,
        cap_percent,
    } = &policy.modifiers[0].change
    else {
        panic!()
    };
    assert_eq!(
        steps,
        &native.skills[0]
            .steps
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!((*increment_percent, *cap_percent), (3, 20));
    assert!(native.skills[1].action_policy.is_none());
    let rate =
        &artifact.sources["skills[0].action_policy.modifiers[0].change.Multihit.increment_percent"];
    assert_eq!(rate.definition, "values[8].value.Count");
    let selection = &artifact.sources["skills[0].action_policy.modifiers[0].change.Multihit.steps"];
    for index in 0..3 {
        assert!(selection
            .inputs
            .contains(&format!("skills[0].body.children[{index}]")));
    }
}

#[test]
fn composition_multihit_rejects_bad_rates_scope_and_expansion_without_publication() {
    for (value, expected) in [
        (json!({"Literal":{"Milliseconds":2}}), DiagnosticCode::Type),
        (json!({"Literal":{"Count":0}}), DiagnosticCode::Native),
        (
            json!({"Literal":{"Count":2147483648u32}}),
            DiagnosticCode::Native,
        ),
        (json!({"Shared":"missing"}), DiagnosticCode::Reference),
        (
            json!({"Parameter":"outside-call"}),
            DiagnosticCode::Reference,
        ),
    ] {
        for field in ["increment_percent", "cap_percent"] {
            let mut doc = document();
            doc["skills"][0]["modifiers"][0]["change"]["Multihit"][field] = value.clone();
            let error = compile_json(&doc.to_string()).unwrap_err();
            assert_eq!(error.code, expected, "{field}: {error:?}");
        }
    }
    let mut doc = document();
    doc["skills"][0]["body"] =
        json!({"Wait":{"id":"no-damage","duration":{"Literal":{"Milliseconds":1}}}});
    let error = compile_json(&doc.to_string()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert!(error.reason.contains("direct damage"));
    for size in [64, 65] {
        let mut doc = document();
        let call = doc["skills"][0]["body"]["Sequence"]["children"][0].clone();
        doc["skills"][0]["body"]["Sequence"]["children"] = Json::Array(
            (0..size)
                .map(|i| {
                    let mut copy = call.clone();
                    copy["Call"]["id"] = json!(format!("hit-{i}"));
                    copy
                })
                .collect(),
        );
        let result = compile_json(&doc.to_string());
        if size == 64 {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.code, DiagnosticCode::Limit);
            assert!(error.reason.contains("64 expanded damage"));
        }
    }
}
