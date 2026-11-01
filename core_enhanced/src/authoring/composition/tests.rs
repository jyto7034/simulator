use super::*;
use serde_json::{json, Value as Json};

pub(crate) const FIXTURE: &str =
    include_str!("../../../tests/fixtures/skill_composition/woodsman_monk.json");

fn source() -> Json {
    serde_json::from_str(FIXTURE).unwrap()
}
fn compile(source: &Json) -> Result<Artifact, Diagnostic> {
    compile_json(&source.to_string())
}

#[test]
fn parse_and_version_failures_are_structured() {
    assert_eq!(compile_json("{}").unwrap_err().code, DiagnosticCode::Parse);
    let error =
        compile_json(r#"{"version":99,"id":"test","values":[],"definitions":[],"skills":[]}"#)
            .unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Version);
    assert_eq!(error.source.definition, "version");
}

#[test]
fn composition_file_roundtrip_loads_two_different_skills_and_preserves_call_origins() {
    let artifact = compile_json(FIXTURE).unwrap();
    let mut loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    assert!(!loaded.is_source_verified());
    loaded.verify_json_source(FIXTURE).unwrap();
    let skills = loaded.load_native().unwrap();
    assert_eq!(skills.skills.len(), 2);
    assert_eq!(skills.skills[0].steps.len(), 3);
    assert_eq!(skills.skills[1].steps.len(), 1);
    for skill in &skills.skills {
        assert!(!skills.compiled_flow(skill.id.as_str()).unwrap().legacy);
        for step in &skill.steps {
            let origin = loaded
                .origin_for(step.effects[0].source().unwrap())
                .unwrap();
            assert_eq!(origin.definition, "definitions[0].body.effects[0]");
            assert_eq!(origin.calls.len(), 3);
            assert!(origin.inputs.iter().any(|p| p.starts_with("values[")));
        }
    }
    assert_ne!(skills.skills[0].steps[0].id, skills.skills[0].steps[1].id);
}

#[test]
fn composition_rejects_missing_references_and_wrong_parameter_units() {
    let mut doc = source();
    doc["skills"][0]["body"]["Sequence"]["children"][0]["Call"]["arguments"][0]["value"] =
        json!({"Shared":"missing"});
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Reference);
    assert!(error.source.definition.ends_with("arguments[0].value"));
    let mut doc = source();
    doc["skills"][0]["body"]["Sequence"]["children"][0]["Call"]["arguments"][3]["value"] =
        json!({"Literal":{"Count":100}});
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Type);
    assert!(error.reason.contains("Milliseconds"));
    assert_eq!(
        error.source.calls,
        ["skills[0]", "skills[0].body.children[0]"]
    );
    assert_eq!(error.source.inputs, ["definitions[1].parameters"]);
}

#[test]
fn composition_unresolved_source_roundtrips_but_does_not_emit_an_artifact() {
    let mut doc = source();
    doc["definitions"][1]["body"]["Sequence"]["children"][0]["Wait"]["duration"] =
        json!({"Unresolved":"밸런스 미정"});
    let model: Module = serde_json::from_value(doc).unwrap();
    let saved = serde_json::to_string(&model).unwrap();
    let reopened: Module = serde_json::from_str(&saved).unwrap();
    let error = compile_json(&serde_json::to_string(&reopened).unwrap()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Unresolved);
    assert_eq!(error.reason, "밸런스 미정");
    assert_eq!(
        error.source.calls,
        ["skills[0]", "skills[0].body.children[0]"]
    );
}

#[test]
fn composition_native_amount_unit_error_keeps_definition_and_call_chain() {
    let mut doc = source();
    doc["values"][0]["value"]["Amount"]["root"] =
        json!({"Const":{"value":5,"unit":"Milliseconds"}});
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native, "{error:?}");
    assert_eq!(error.source.definition, "definitions[0].body.effects[0]");
    assert_eq!(error.source.calls.len(), 3);
    assert!(error.source.inputs.contains(&"values[0].value".into()));
}

#[test]
fn composition_rejects_recursive_unknown_and_unsupported_definitions() {
    let mut doc = source();
    doc["definitions"][0]["body"] = json!({"Call":{"id":"loop","definition":"area-impact","arguments":[
        {"name":"power","value":{"Parameter":"power"}},
        {"name":"delivery","value":{"Parameter":"delivery"}},
        {"name":"after","value":{"Parameter":"after"}}
    ]}});
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Limit);
    let mut doc = source();
    doc["definitions"][1]["body"]["Sequence"]["children"][1]["Call"]["definition"] =
        json!("missing");
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Reference);
    let mut doc = source();
    doc["definitions"][0]["body"] =
        json!({"Unsupported":{"id":"external","operation":"external-code"}});
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Unsupported);
}

#[test]
fn composition_rejects_duplicate_names_and_missing_or_extra_inputs() {
    let mut doc = source();
    let duplicate = doc["values"][0].clone();
    doc["values"].as_array_mut().unwrap().push(duplicate);
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Reference);
    for mode in ["missing", "extra", "duplicate"] {
        let mut doc = source();
        let args = doc["skills"][1]["body"]["Call"]["arguments"]
            .as_array_mut()
            .unwrap();
        match mode {
            "missing" => {
                args.pop();
            }
            "extra" => args.push(json!({"name":"unused","value":{"Literal":{"Count":1}}})),
            _ => args.push(args[0].clone()),
        }
        assert_eq!(
            compile(&doc).unwrap_err().code,
            DiagnosticCode::Reference,
            "{mode}"
        );
    }
}

#[test]
fn composition_rejects_artifact_version_and_invalid_native_payload() {
    let artifact = compile_json(FIXTURE).unwrap();
    let mut wrong_version = artifact.clone();
    wrong_version.version = 99;
    assert_eq!(
        wrong_version.load_native().unwrap_err().code,
        DiagnosticCode::Version
    );
    let mut invalid = artifact;
    invalid.native_ron = "(skills: []) trailing".into();
    assert_eq!(
        invalid.load_native().unwrap_err().code,
        DiagnosticCode::Native
    );
}

fn branch_after_body(doc: &mut Json, metric: &str, value: Json) {
    doc["definitions"][0]["body"]["Action"]["wait"] = json!("EffectsSettled");
    let first = doc["skills"][0]["body"].clone();
    doc["skills"][0]["body"] = json!({"Sequence":{"id":"conditional","children":[first,
        {"Branch":{"id":"decision","condition":{"Compare":{"node_id":"three-strikes","metric":metric,"comparison":"AtLeast","value":value}},
            "then_node":{"Wait":{"id":"yes","duration":{"Literal":{"Milliseconds":0}}}},
            "else_node":{"Wait":{"id":"no","duration":{"Literal":{"Milliseconds":0}}}}
        }}
    ]}});
}

#[test]
fn composition_conditions_validate_local_result_reference_units_and_settlement() {
    let mut valid = source();
    branch_after_body(&mut valid, "DamagedTargets", json!({"Count":1}));
    compile(&valid).unwrap();
    let mut bad = valid.clone();
    bad["skills"][0]["body"]["Sequence"]["children"][1]["Branch"]["condition"]["Compare"]
        ["node_id"] = json!("hit");
    let error = compile(&bad).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Reference);
    assert!(error.reason.contains("local scope"));
    let mut bad = valid.clone();
    bad["skills"][0]["body"]["Sequence"]["children"][1]["Branch"]["condition"]["Compare"]
        ["value"] = json!({"Hp":1});
    let error = compile(&bad).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Type);
    assert!(error.source.definition.ends_with("condition.value"));
    let mut bad = valid;
    bad["definitions"][0]["body"]["Action"]["wait"] = json!("Released");
    let error = compile(&bad).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert!(error.reason.contains("not EffectsSettled"), "{error:?}");
}

#[test]
fn composition_attachment_reference_is_validated_after_arguments_are_bound() {
    let mut doc = source();
    doc["values"][5]["value"]["Effects"][0]["attached_to"] = json!("absent-damage");
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert!(
        error
            .source
            .definition
            .contains("definitions[0].body.effects[1]"),
        "{error:?}"
    );
    assert_eq!(error.source.calls.len(), 3);
    assert!(error.source.inputs.contains(&"values[5].value".into()));
}

#[test]
fn composition_unused_and_called_definitions_validate_local_condition_references_and_units() {
    for called in [false, true] {
        for effect_specific in [false, true] {
            let mut doc = source();
            let mut action = doc["definitions"][0]["body"].clone();
            action["Action"]["wait"] = json!("EffectsSettled");
            action["Action"]["delivery"] = json!({"Literal":{"Delivery":"Instant"}});
            action["Action"]["effects"] = json!([{"Damage":{
                "id":"damage", "tags":[], "amount":{"Shared":"weapon-power"}, "damage_type":"Physical"
            }}]);
            let variant = if effect_specific {
                "EffectCompare"
            } else {
                "Compare"
            };
            let mut comparison = json!({"node_id":"hit", "metric":"HpLost", "comparison":"AtLeast", "value":{"Hp":1}});
            if effect_specific {
                comparison["effect_id"] = json!("damage");
            }
            let index = doc["definitions"].as_array().unwrap().len();
            doc["definitions"].as_array_mut().unwrap().push(json!({
                "name":"library-condition", "parameters":[],
                "body":{"Sequence":{"id":"test","children":[action,
                    {"Branch":{"id":"decision", "condition":{variant:comparison},
                        "then_node":{"Wait":{"id":"yes","duration":{"Literal":{"Milliseconds":0}}}},
                        "else_node":{"Wait":{"id":"no","duration":{"Literal":{"Milliseconds":0}}}}
                    }}
                ]}}
            }));
            if called {
                doc["skills"][0]["body"] =
                    json!({"Call":{"id":"invoke","definition":"library-condition","arguments":[]}});
            }
            compile(&doc).expect("valid library condition is accepted unused and called");
            for (field, value, code) in [
                ("node_id", json!("absent"), DiagnosticCode::Reference),
                ("value", json!({"Count":1}), DiagnosticCode::Type),
            ] {
                let mut bad = doc.clone();
                bad["definitions"][index]["body"]["Sequence"]["children"][1]["Branch"]
                    ["condition"][variant][field] = value;
                let error = compile(&bad).unwrap_err();
                assert_eq!(error.code, code);
                assert_eq!(
                    error.source.definition,
                    format!("definitions[{index}].body.children[1].condition.{field}")
                );
                assert_eq!(error.source.calls.is_empty(), !called);
            }
        }
    }
}

#[test]
fn composition_repeat_limit_diagnostic_keeps_shared_input_origin() {
    let mut doc = source();
    let index = doc["values"].as_array().unwrap().len();
    doc["values"].as_array_mut().unwrap().push(json!({
        "name":"too-many", "value":{"Count":1_000_001}
    }));
    let body = doc["skills"][0]["body"].clone();
    doc["skills"][0]["body"] = json!({"Repeat":{
        "id":"repeated", "count":{"Shared":"too-many"},
        "interval":{"Literal":{"Milliseconds":1}},
        "body":body
    }});
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert_eq!(error.source.definition, "skills[0].body.count");
    assert_eq!(error.source.calls, ["skills[0]"]);
    assert!(error
        .source
        .inputs
        .contains(&format!("values[{index}].value")));
}

#[test]
fn composition_expansion_is_bounded_without_unrolling_runtime_repeat() {
    let mut repeated = source();
    let body = repeated["skills"][0]["body"].clone();
    repeated["skills"][0]["body"] = json!({"Repeat":{"id":"many","count":{"Literal":{"Count":1000000}},"interval":{"Literal":{"Milliseconds":1}},"body":body}});
    let artifact = compile(&repeated).unwrap();
    let database = artifact.load_native().unwrap();
    assert_eq!(
        database.skills[0].steps.len(),
        3,
        "repeat keeps a compact immutable definition"
    );
    repeated["skills"][0]["body"]["Repeat"]["count"] = json!({"Literal":{"Count":1000001}});
    assert_eq!(compile(&repeated).unwrap_err().code, DiagnosticCode::Native);

    let mut expanded = source();
    let mut last = "leaf".to_string();
    let leaf = expanded["skills"][0]["body"].clone();
    expanded["definitions"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":last,"parameters":[],"body":leaf}));
    for i in 0..10 {
        let name = format!("double-{i}");
        expanded["definitions"].as_array_mut().unwrap().push(
            json!({"name":name,"parameters":[],"body":{"Parallel":{"id":"pair","branches":[
                {"Call":{"id":"a","definition":last,"arguments":[]}},
                {"Call":{"id":"b","definition":last,"arguments":[]}}
            ]}}}),
        );
        last = name;
    }
    expanded["skills"][0]["body"] =
        json!({"Call":{"id":"expanded","definition":last,"arguments":[]}});
    let error = compile(&expanded).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Limit);
    assert!(error.reason.contains("expanded flow node"), "{error:?}");
}

#[test]
fn composition_input_errors_preserve_parse_boundary_and_source_identity() {
    let duplicate = FIXTURE.replacen("\"version\": 1", "\"version\": 1, \"version\": 1", 1);
    assert_eq!(
        compile_json(&duplicate).unwrap_err().code,
        DiagnosticCode::Parse
    );
    let mut trailing = FIXTURE.to_string();
    trailing.push_str(" true");
    assert_eq!(
        compile_json(&trailing).unwrap_err().code,
        DiagnosticCode::Parse
    );
    assert_eq!(
        compile_json(&" ".repeat(MAX_INPUT_BYTES + 1))
            .unwrap_err()
            .code,
        DiagnosticCode::Limit
    );
    let artifact = compile_json(FIXTURE).unwrap();
    let other = compile_json(&format!("{FIXTURE} ")).unwrap();
    let db = artifact.load_native().unwrap();
    let effect = db.skills[0].steps[0].effects[0].source().unwrap();
    assert!(artifact.origin_for(effect).is_some());
    assert!(other.origin_for(effect).is_none());
}

#[test]
fn personal_shield_definition_reuses_typed_inputs_and_preserves_origins() {
    let yaml = include_str!("../../../tests/fixtures/skill_composition/personal_shield.skill.yaml");
    let mut doc = parse_yaml(yaml).unwrap();
    let mut second = doc["skills"][0].clone();
    second["id"] = json!("alternate-protection");
    second["body"]["Call"]["arguments"][0]["value"] = json!({"Literal":{"Amount":{
        "moment":"Snapshot", "root":{"Const":{"value":75,"unit":"Magnitude"}}
    }}});
    second["body"]["Call"]["arguments"][1]["value"] = json!({"Literal":{"Milliseconds":900}});
    doc["skills"].as_array_mut().unwrap().push(second);
    let artifact = compile(&doc).unwrap();
    let mut artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    artifact.verify_json_source(&doc.to_string()).unwrap();
    let native = artifact.load_native().unwrap();
    for (index, duration) in [6000, 900].into_iter().enumerate() {
        let skill = &native.skills[index];
        assert_eq!(
            skill.use_lifetime,
            crate::game::ability::SkillUseLifetime::Action
        );
        let effect = &skill.steps[0].effects[0];
        let crate::game::ability::SkillEffectDef::PersonalShieldGrant { definition } = &effect.op
        else {
            panic!("native shield consumer missing")
        };
        assert_eq!(definition.duration_ms, duration);
        assert!(definition.replace_only_recipients);
        let origin = artifact.origin_for(effect.source().unwrap()).unwrap();
        assert_eq!(origin.definition, "definitions[0].body.effects[0]");
        assert!(origin
            .calls
            .iter()
            .any(|p| p == &format!("skills[{index}].body")));
        assert!(origin.inputs.iter().any(|p| p.contains("arguments[0]")));
        assert!(origin.inputs.iter().any(|p| p.contains("arguments[1]")));
    }
    let first = &native.skills[0].steps[0].effects[0].op;
    let second = &native.skills[1].steps[0].effects[0].op;
    assert_ne!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
}

#[test]
fn personal_shield_inputs_reject_wrong_units_overflow_and_unresolved_values() {
    let yaml = include_str!("../../../tests/fixtures/skill_composition/personal_shield.skill.yaml");
    for (value, code) in [
        (json!({"Literal":{"Count":100}}), DiagnosticCode::Type),
        (
            json!({"Literal":{"Milliseconds":u64::from(u32::MAX)+1}}),
            DiagnosticCode::Type,
        ),
        (
            json!({"Unresolved":"지속 시간 미정"}),
            DiagnosticCode::Unresolved,
        ),
    ] {
        let mut doc = parse_yaml(yaml).unwrap();
        doc["skills"][0]["body"]["Call"]["arguments"][1]["value"] = value;
        let error = compile(&doc).unwrap_err();
        assert_eq!(error.code, code, "{error:?}");
        assert!(!error.source.calls.is_empty(), "{error:?}");
    }
    let mut doc = parse_yaml(yaml).unwrap();
    doc["values"][0]["value"]["Amount"]["root"] = json!({"Const":{"value":5,"unit":"Count"}});
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert_eq!(error.source.definition, "definitions[0].body.effects[0]");
    assert!(error.source.inputs.contains(&"values[0].value".into()));
}

#[test]
fn personal_shield_repeated_metadata_is_charged_before_expansion() {
    let mut doc = parse_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/personal_shield.skill.yaml"
    ))
    .unwrap();
    doc["definitions"][0]["body"]["Action"]["effects"][0]["PersonalShieldGrant"]["tags"] =
        json!(["x".repeat(100_000)]);
    let call = doc["skills"][0]["body"].clone();
    let calls: Vec<_> = (0..100)
        .map(|i| {
            let mut call = call.clone();
            call["Call"]["id"] = json!(format!("grant-{i}"));
            call
        })
        .collect();
    doc["skills"][0]["body"] = json!({"Sequence":{"id":"many-grants","children":calls}});
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Limit, "{error:?}");
    assert_eq!(error.reason, "expanded payload byte limit exceeded");
}

#[test]
fn composition_manual_state_keeps_authored_child_flow_and_shared_origin() {
    use crate::game::ability::{state_program::*, sustained::SustainedEffectDef, SkillEffectDef};
    let artifact = compile_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/manual_state.skill.yaml"
    ))
    .unwrap();
    let native = Artifact::from_ron(&artifact.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let parent = &native.skills[0];
    let effect = &parent.steps[0].effects[0];
    let SkillEffectDef::Sustained {
        definition: SustainedEffectDef::State { program, .. },
    } = &effect.op
    else {
        panic!("authored state");
    };
    assert!(program.has_manual_input());
    assert_eq!(
        program
            .basic_attack
            .as_ref()
            .unwrap()
            .line
            .as_ref()
            .unwrap()
            .power,
        40
    );
    assert!(
        matches!(&program.handlers[0].body[0], StateOp::EndAndInvoke { skill } if skill.as_str() == "authored-finish")
    );
    let origin = artifact.origin_for(effect.source().unwrap()).unwrap();
    assert!(origin.inputs.iter().any(|s| s.starts_with("values[0]")));
    assert!(native.skills[1].flow.is_some());
    assert!(!native.compiled_flow("authored-finish").unwrap().legacy);
}

#[test]
fn composition_manual_state_rejects_missing_child_cost_context_and_complex_flow() {
    let original = parse_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/manual_state.skill.yaml"
    ))
    .unwrap();
    for case in [
        "missing",
        "self",
        "cost",
        "focus",
        "flow",
        "expired",
        "duplicate",
        "independent",
        "overlay-flag",
        "unreachable",
    ] {
        let mut doc = original.clone();
        let program = &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
            ["State"]["program"];
        match case {
            "missing" => {
                program["handlers"][0]["body"][0]["EndAndInvoke"]["skill"] = json!("missing")
            }
            "self" => {
                program["handlers"][0]["body"][0]["EndAndInvoke"]["skill"] =
                    json!("authored-manual-state")
            }
            "expired" => program["handlers"][0]["event"] = json!("Expired"),
            "duplicate" => {
                let handler = program["handlers"][0].clone();
                program["handlers"].as_array_mut().unwrap().push(handler);
            }
            "independent" => program["owns_use"] = json!(false),
            "overlay-flag" => program["basic_attack"]["followup_on_input"] = json!(true),
            "unreachable" => program["handlers"][0]["body"]
                .as_array_mut()
                .unwrap()
                .push(json!("End")),
            "cost" => doc["skills"][1]["admission"]["cost"] = json!("All"),
            "focus" => doc["skills"][1]["prepare_ms"] = json!(1),
            "flow" => {
                let body = doc["skills"][1]["body"].clone();
                doc["skills"][1]["body"] =
                    json!({"Sequence":{"id":"extra-flow", "children":[body]}});
            }
            _ => unreachable!(),
        }
        let error = compile(&doc).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Native, "{case}: {error:?}");
        assert!(!error.reason.is_empty());
        assert!(error
            .source
            .definition
            .starts_with("values[0].value.Effects[0].op.Sustained"));
        assert!(error
            .source
            .inputs
            .contains(&"definitions[0].body.effects[0]".into()));
        assert!(!error.source.calls.is_empty());
    }
}

#[test]
fn composition_appended_native_error_selects_the_literal_item_across_parameters() {
    let original = parse_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/manual_state.skill.yaml"
    ))
    .unwrap();
    for (inline, nested) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut doc = original.clone();
        let value = &mut doc["values"][0]["value"];
        value["Effects"][0]["op"]["Sustained"]["definition"]["State"]["program"]["handlers"][0]
            ["body"][0]["EndAndInvoke"]["skill"] = json!("missing");
        if nested {
            let program =
                &mut value["Effects"][0]["op"]["Sustained"]["definition"]["State"]["program"];
            let child = program["handlers"][0]["body"][0].clone();
            program["fields"] = json!([{ "name":"ready", "value":{"Bool":true} }]);
            program["handlers"][0]["body"] = json!([{"If":{
                "condition":{"Bool":{"field":"ready","equals":true}},
                "then_ops":[child],"else_ops":["End"]
            }}]);
        }
        value["Effects"].as_array_mut().unwrap().insert(
            0,
            json!({"id":"other-state", "op":{"Sustained":{"definition":{"State":{
                "duration_ms":100, "program":{"id":"other-state", "owns_use":false,
                    "end_on_source_exit":true, "replacement":"ReplaceSameSource", "fields":[],
                    "handlers":[{"id":"expiry", "event":"Expired", "body":["End"]}]}
            }}}}}),
        );
        if inline {
            doc["skills"][0]["body"]["Call"]["arguments"][0]["value"] =
                json!({"Literal": doc["values"][0]["value"]});
        }
        let error = compile(&doc).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Native, "{error:?}");
        assert!(
            error.reason.contains("EndAndInvoke child 'missing'"),
            "{error:?}"
        );
        let prefix = if inline {
            "skills[0].body.arguments[0].value.Literal"
        } else {
            "values[0].value"
        };
        let nested_path = if nested { ".If.then_ops[0]" } else { "" };
        assert_eq!(error.source.definition, format!("{prefix}.Effects[1].op.Sustained.State.program.handlers[0].body[0]{nested_path}.EndAndInvoke.skill"));
    }
}

const RESOURCE_SOURCE: &str =
    include_str!("../../../tests/fixtures/skill_composition/resources.skill.yaml");

#[test]
fn composition_resource_declarations_roundtrip_and_validate_operations_and_reads() {
    let artifact = compile_yaml(RESOURCE_SOURCE).unwrap();
    let reopened = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    assert_eq!(reopened.resources(), artifact.resources());
    assert_eq!(reopened.resources().len(), 3);
    assert!(reopened.load_native().unwrap().skills[0]
        .resonance_admission
        .is_none());
    for (before, after, field) in [
        ("initial: 7", "initial: 16", "resources[0].initial"),
        ("id: copy_meter", "id: employee_meter", "resources[1].id"),
        (
            "resource: copy_meter",
            "resource: absent",
            "values[0].value.Effects[0].op.Resources.operations[1].resource",
        ),
        (
            "owner: SourceEquipment",
            "owner: SourceEmployee",
            "values[0].value.Effects[0].op.Resources.operations[1].owner",
        ),
    ] {
        let error = compile_yaml(&RESOURCE_SOURCE.replacen(before, after, 1)).unwrap_err();
        assert_eq!(error.source.definition, field, "{error:?}");
    }
    let error = compile_yaml(
        &RESOURCE_SOURCE
            .replace(
                "Resource: { resource: fragment_meter",
                "Resource: { resource: absent",
            )
            .replace("Resource: fragment_meter", "Resource: absent"),
    )
    .unwrap_err();
    assert!(
        error.reason.contains("unknown value expression resource"),
        "{error:?}"
    );
    for invalid in [
        RESOURCE_SOURCE.replace("lifetime: Run", "lifetime: Forever"),
        RESOURCE_SOURCE.replace("initial: 7", "initial: -1"),
        RESOURCE_SOURCE.replacen("    admission: null\n", "", 1),
    ] {
        assert_eq!(
            compile_yaml(&invalid).unwrap_err().code,
            DiagnosticCode::Parse
        );
    }
    // The envelope is not trusted merely because it was previously compiled.
    let mut tampered = artifact;
    tampered.resources[0].cap = 0;
    assert!(tampered.load_native().is_err());
}

#[test]
fn composition_state_resource_nested_declarations_and_limits_are_checked() {
    let source =
        include_str!("../../../tests/fixtures/skill_composition/state_resources.skill.yaml");
    let artifact = compile_yaml(source).unwrap();
    assert_eq!(
        Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .resources(),
        artifact.resources()
    );
    for branch in ["then_ops", "else_ops"] {
        for (key, value, reason) in [
            ("resource", "absent", "unknown resource"),
            ("owner", "SourceEmployee", "selector owner kind differs"),
        ] {
            let mut doc = parse_yaml(source).unwrap();
            let block = &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]
                ["definition"]["State"]["program"]["handlers"][0]["body"][1]["If"];
            if branch == "else_ops" {
                block["else_ops"] = block["then_ops"].take();
                block["then_ops"] = json!([]);
            }
            block[branch][0]["Resources"]["operations"][0][key] = json!(value);
            let error = compile(&doc).unwrap_err();
            assert!(error.reason.contains(reason), "{error:?}");
            assert_eq!(error.source.definition, format!("values[0].value.Effects[0].op.Sustained.definition.State.program.handlers[0].body[1].If.{branch}[0].Resources.operations[0].{key}"));
        }
    }
    for count in [
        0,
        crate::game::resource::store::ResourceLimits::default().batch + 1,
    ] {
        let mut doc = parse_yaml(source).unwrap();
        let ops = &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
            ["State"]["program"]["handlers"][0]["body"][1]["If"]["then_ops"][0]["Resources"]
            ["operations"];
        let op = ops[0].clone();
        *ops = json!(vec![op; count]);
        assert!(compile(&doc).is_err(), "batch {count}");
    }
}

#[test]
fn integration_source_origin_rejects_mixed_effect_identity_and_unknown_field() {
    use crate::game::ability::authoring::EffectOrigin;
    let artifact = compile_json(FIXTURE).unwrap();
    let native = artifact.load_native().unwrap();
    let original = native.skills[0].steps[0].effects[0]
        .source()
        .unwrap()
        .as_ref();
    // Expected relation comes from the controlled authored fixture, not from
    // comparing two renderings of its generated source metadata.
    assert_eq!(
        original.site.action.definition_id.as_str(),
        "woodsman-authored"
    );
    assert_eq!(original.field_path, "skills[0].steps[0].effects[0]");
    assert_eq!(
        artifact.origin_for(original).unwrap().definition,
        "definitions[0].body.effects[0]"
    );
    let mut invalid = Vec::new();
    let mut source = original.clone();
    source.site.action.definition_id = "monk-authored".into();
    invalid.push(("other_definition", source));
    let mut source = original.clone();
    source.site.action.action_id = native.skills[0].steps[1].id.clone();
    invalid.push(("other_action", source));
    let mut source = original.clone();
    source.site.effect_id = "absent_effect".into();
    invalid.push(("absent_effect", source));
    let mut source = original.clone();
    source.origin_definition_id = "monk-authored".into();
    invalid.push(("other_origin_definition", source));
    let mut source = original.clone();
    source.origin = EffectOrigin::Added {
        source_definition_id: "monk-authored".into(),
    };
    invalid.push(("wrong_origin_kind", source));
    let mut source = original.clone();
    source.field_path.push_str(".not_a_native_field");
    invalid.push(("unknown_field_suffix", source));
    let mut source = original.clone();
    source.field_path.push_str(".id");
    invalid.push(("existing_field_is_not_runtime_producer", source));
    let mut source = original.clone();
    source.field_path = "skills[1].steps[0].effects[0]".into();
    invalid.push(("other_skill_field", source));
    let other = compile_json(&format!("{FIXTURE} ")).unwrap();
    assert!(
        other.origin_for(original).is_none(),
        "different original bytes are a different source"
    );
    let accepted: Vec<_> = invalid
        .iter()
        .filter_map(|(label, source)| artifact.origin_for(source).is_some().then_some(*label))
        .collect();
    assert!(
        accepted.is_empty(),
        "source lookup accepted inconsistent provenance: {accepted:?}"
    );
}

#[test]
fn integration_imported_source_map_requires_original_before_attribution() {
    let artifact = compile_json(FIXTURE).unwrap();
    let native = artifact.load_native().unwrap();
    let source = native.skills[0].steps[0].effects[0].source().unwrap();
    let mut encoded = serde_json::to_value(&artifact).unwrap();
    let path = "skills[0].steps[0].effects[0]";
    encoded["sources"][path]["definition"] = json!("definitions[1].body");
    let mut modified: Artifact = serde_json::from_value(encoded).unwrap();
    assert!(modified.load_native().is_ok(), "executable data is unchanged");
    assert!(modified.origin_for(source).is_none(),
        "unverified imported source map must not attest a forged original location");
    assert!(modified.verify_json_source(FIXTURE).is_err());
    assert!(!modified.is_source_verified());
    assert!(modified.origin_for(source).is_none());
    let mut valid = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    assert!(valid.origin_for(source).is_none());
    valid.verify_json_source(FIXTURE).unwrap();
    assert_eq!(valid.origin_for(source).unwrap().definition, "definitions[0].body.effects[0]");
    let wrong_version = format!("{FIXTURE} ");
    assert!(valid.verify_json_source(&wrong_version).is_err());
    assert!(!valid.is_source_verified());
    assert!(valid.origin_for(source).is_none());
    valid.verify_json_source(FIXTURE).unwrap();
    assert_eq!(valid.origin_for(source).unwrap().definition, "definitions[0].body.effects[0]");
}

#[test]
fn integration_yaml_source_verification_rejects_missing_map_and_unrelated_calls() {
    let yaml = include_str!("../../../tests/fixtures/skill_composition/woodsman_monk.skill.yaml");
    let original = compile_yaml(yaml).unwrap();
    let native = original.load_native().unwrap();
    let source = native.skills[0].steps[0].effects[0].source().unwrap();
    let path = "skills[0].steps[0].effects[0]";
    let mut valid = Artifact::from_ron(&original.to_ron().unwrap()).unwrap();
    assert!(valid.origin_for(source).is_none());
    valid.verify_yaml_source(yaml).unwrap();
    assert_eq!(valid.origin_for(source).unwrap().definition, "definitions[0].body.effects[0]");
    for kind in ["missing", "calls", "inputs", "digest"] {
        let mut encoded = serde_json::to_value(&original).unwrap();
        match kind {
            "missing" => { encoded["sources"].as_object_mut().unwrap().remove(path); }
            "calls" => encoded["sources"][path]["calls"] = json!(["skills[1]"]),
            "inputs" => encoded["sources"][path]["inputs"] = json!(["values[999]"]),
            "digest" => encoded["source_digest"] = json!("0".repeat(64)),
            _ => unreachable!(),
        }
        let mut imported: Artifact = serde_json::from_value(encoded).unwrap();
        assert!(imported.load_native().is_ok(), "{kind}: executable data unchanged");
        assert!(imported.origin_for(source).is_none());
        assert!(imported.verify_yaml_source(yaml).is_err(), "{kind}");
        assert!(!imported.is_source_verified());
        assert!(imported.origin_for(source).is_none());
    }
}

#[test]
fn integration_unverified_artifact_error_keeps_native_address_not_claimed_source_map() {
    let original = compile_json(FIXTURE).unwrap();
    let mut encoded = serde_json::to_value(&original).unwrap();
    encoded["native_ron"] = json!(original.native_ron.replace("monk-authored", "woodsman-authored"));
    for trace in encoded["sources"].as_object_mut().unwrap().values_mut() {
        trace["definition"] = json!("forged.authoring.location");
    }
    let imported: Artifact = serde_json::from_value(encoded).unwrap();
    assert!(!imported.is_source_verified());
    let error = imported.load_native().unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert_eq!(error.source.definition, "skills[1].id", "{error:?}");
    assert!(error.source.calls.is_empty() && error.source.inputs.is_empty());
}


#[test]
fn integration_authored_status_cannot_fabricate_runtime_modifier_source() {
    // Identity fields are runtime observations of an actually equipped copy.
    // Native literals are supported authoring input, but cannot assert that a
    // made-up inventory copy produced an effect.
    let claimed = json!({
        "copy_uuid": "00000000-0000-0000-0000-00000000f00d",
        "definition": "never-equipped",
        "rule": "never-resolved"
    });
    let mut area = parse_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/control_vulnerability.skill.yaml"
    ))
    .unwrap();
    area["definitions"][0]["body"]["Action"]["modifiers"] = json!([]);
    area["skills"][0]["modifiers"] = json!([]);
    let status = &mut area["values"][0]["value"]["Delivery"]["TileArea"]["area"]["ground_status"]
        ["statuses"][1];
    status["vulnerability"] =
        json!({"definition":"native-weakness","percent":15,"duration_ms":5000});
    compile_json(&area.to_string()).expect("normal native ground vulnerability");
    area["values"][0]["value"]["Delivery"]["TileArea"]["area"]["ground_status"]["statuses"][1]
        ["vulnerability"]["modifier_source"] = claimed.clone();

    let mut direct = parse_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/freeze_vulnerability.skill.yaml"
    ))
    .unwrap();
    direct["definitions"][0]["body"]["Action"]["modifiers"] = json!([]);
    compile_json(&direct.to_string()).expect("normal native freeze");
    let mut nested = direct.clone();
    nested["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["HitStatus"]
        ["profile"]["vulnerability"] =
        json!({"definition":"native-weakness","percent":15,"duration_ms":5000});
    compile_json(&nested.to_string()).expect("normal native freeze vulnerability");
    direct["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["HitStatus"]
        ["profile"]["modifier_source"] = claimed.clone();
    nested["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["HitStatus"]
        ["profile"]["vulnerability"]["modifier_source"] = claimed;
    let accepted: Vec<_> = [
        ("ground_vulnerability", area),
        ("direct_status", direct),
        ("direct_vulnerability", nested),
    ]
    .into_iter()
    .filter_map(
        |(label, document)| match compile_json(&document.to_string()) {
            Ok(_) => Some(label),
            Err(error) => {
                assert!(
                    error.reason.contains("modifier_source"),
                    "must reject runtime provenance, not a different error: {error:?}"
                );
                None
            }
        },
    )
    .collect();
    assert!(
        accepted.is_empty(),
        "authoring accepted fabricated runtime sources: {accepted:?}"
    );
}

#[test]
fn integration_runtime_source_boundary_covers_equipment_state_native_and_snapshots() {
    use crate::game::{
        ability::{
            sustained::{
                HitModifierSource, HitStatusKind, HitStatusProfile, HitVulnerability,
                SustainedEffectDef,
            },
            DeliveryDef, SkillEffectDef,
        },
        data::skill_data::SkillDatabase,
    };
    let claim = HitModifierSource {
        copy_uuid: uuid::Uuid::from_u128(0xf00d),
        definition: "never-equipped".into(),
        rule: "never-resolved".into(),
    };
    // Diagnostic/runtime data still roundtrips. Parsing an observation does not
    // make it an authored producer and must not silently erase its identity.
    let observed = HitStatusProfile {
        definition: "observed-freeze".into(),
        kind: HitStatusKind::Freeze,
        modifier_source: Some(claim.clone()),
        vulnerability: Some(HitVulnerability {
            definition: "observed-weakness".into(),
            percent: 15,
            duration_ms: 5000,
            modifier_source: Some(claim.clone()),
        }),
    };
    let restored: HitStatusProfile =
        ron::de::from_str(&ron::ser::to_string(&observed).unwrap()).unwrap();
    assert_eq!(restored, observed);
    restored
        .validate()
        .expect("runtime observation remains structurally valid");

    for (text, pointer) in [
        (
            include_str!("../../../tests/fixtures/skill_composition/chance_stun.skill.yaml"),
            "/values/0/value/AttackProgram/on_hit_status/profile",
        ),
        (
            include_str!("../../../tests/fixtures/skill_composition/equipment_stun_end.skill.yaml"),
            "/values/0/value/StateProgram/handlers/0/body/0/ApplyEventTargetStatus/profile",
        ),
    ] {
        let original = parse_yaml(text).unwrap();
        let good = compile_json(&original.to_string()).expect("normal producer");
        good.load_native().unwrap();
        for nested in [false, true] {
            let mut doc = original.clone();
            let profile = doc.pointer_mut(pointer).unwrap();
            if nested {
                profile["kind"] = json!("Stun");
                profile["vulnerability"] =
                    json!({"definition":"native-weakness","percent":15,"duration_ms":5000});
            }
            compile_json(&doc.to_string()).expect("normal neighboring vulnerability");
            let profile = doc.pointer_mut(pointer).unwrap();
            if nested {
                profile["kind"] = json!("Stun");
                profile["vulnerability"]["modifier_source"] = json!(claim);
            } else {
                profile["modifier_source"] = json!(claim);
            }
            let error = compile_json(&doc.to_string()).unwrap_err();
            assert!(error.reason.contains("modifier_source"), "{error:?}");
            assert_eq!(error.code, DiagnosticCode::Native);
            compile_json(&original.to_string()).expect("rejection leaves a normal retry valid");
        }
        // Serialized native artifacts enter the same definition compilers;
        // source JSON validation is not the sole protection.
        let mut artifact = good.clone();
        if let Some(binding) = artifact.equipment_attacks.first_mut() {
            binding
                .program
                .on_hit_status
                .as_mut()
                .unwrap()
                .profile
                .modifier_source = Some(claim.clone());
        } else {
            let program = &mut artifact.equipment_states[0].program;
            let crate::game::ability::state_program::StateOp::ApplyEventTargetStatus {
                profile,
                ..
            } = &mut program.handlers[0].body[0]
            else {
                panic!("fixture must exercise state target status")
            };
            profile.modifier_source = Some(claim.clone());
        }
        let imported: Artifact =
            ron::de::from_str(&ron::ser::to_string(&artifact).unwrap()).unwrap();
        let error = imported.load_native().unwrap_err();
        assert!(error.reason.contains("modifier_source"), "{error:?}");
        good.load_native().expect("native retry remains valid");
    }

    // Direct native skill catalogs are another supported input, including an
    // otherwise valid status nested in a ground subscription.
    for (text, ground) in [
        (
            include_str!(
                "../../../tests/fixtures/skill_composition/freeze_vulnerability.skill.yaml"
            ),
            false,
        ),
        (
            include_str!(
                "../../../tests/fixtures/skill_composition/control_vulnerability.skill.yaml"
            ),
            true,
        ),
    ] {
        let mut doc = parse_yaml(text).unwrap();
        doc["definitions"][0]["body"]["Action"]["modifiers"] = json!([]);
        doc["skills"][0]["modifiers"] = json!([]);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let mut native = artifact.load_native().unwrap();
        let normal = ron::ser::to_string(&native).unwrap();
        SkillDatabase::from_ron("native-source.ron", &normal).expect("native neighbor");
        let step = &mut native.skills[0].steps[0];
        if ground {
            let DeliveryDef::TileArea { area, .. } = &mut step.delivery else {
                panic!()
            };
            area.ground_status.as_mut().unwrap().statuses[1].vulnerability =
                Some(HitVulnerability {
                    definition: "native-weakness".into(),
                    percent: 15,
                    duration_ms: 5000,
                    modifier_source: None,
                });
        }
        SkillDatabase::from_ron("native-source.ron", &ron::ser::to_string(&native).unwrap())
            .expect("native vulnerability neighbor");
        let step = &mut native.skills[0].steps[0];
        if ground {
            let DeliveryDef::TileArea { area, .. } = &mut step.delivery else {
                panic!()
            };
            area.ground_status.as_mut().unwrap().statuses[1]
                .vulnerability
                .as_mut()
                .unwrap()
                .modifier_source = Some(claim.clone());
        } else {
            let SkillEffectDef::Sustained {
                definition: SustainedEffectDef::HitStatus { profile, .. },
            } = &mut step.effects[0].op
            else {
                panic!()
            };
            profile.modifier_source = Some(claim.clone());
        }
        let error =
            SkillDatabase::from_ron("native-source.ron", &ron::ser::to_string(&native).unwrap())
                .unwrap_err();
        assert!(
            format!("{error:?}").contains("modifier_source"),
            "{error:?}"
        );
        SkillDatabase::from_ron("native-source.ron", &normal).expect("native retry");
    }
}
