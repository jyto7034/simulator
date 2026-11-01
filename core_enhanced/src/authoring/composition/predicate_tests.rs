use super::*;
use serde_json::{json, Value as Json};
fn parameter(name: &str) -> Json {
    json!({"Parameter":name})
}
fn literal(value: Json) -> Json {
    json!({"Literal":value})
}
fn amount(value: i64, unit: &str) -> Json {
    json!({"Amount":{"moment":"Live","root":{"Const":{"value":value,"unit":unit}}}})
}
fn compile(doc: &Json) -> Result<Artifact, Diagnostic> {
    compile_json(&doc.to_string())
}
fn call(doc: &mut Json) -> &mut Json {
    &mut doc["skills"][0]["body"]["Sequence"]["children"][1]["Branch"]["condition"]["Predicate"]
}
fn flow_module() -> Json {
    let mut doc: Json = serde_json::from_str(super::tests::FIXTURE).unwrap();
    doc["version"] = json!(2);
    doc["values"] = json!([]);
    doc["definitions"] = json!([]);
    doc["skills"].as_array_mut().unwrap().truncate(1);
    doc["predicates"] = json!([{"name":"positive-result","context":"Flow","parameters":[
        {"name":"result","value_type":{"FlowResult":"Magnitude"}},{"name":"minimum","value_type":{"FlowLiteral":"Magnitude"}}],
        "body":{"Logic":{"All":[{"Result":{"result":parameter("result"),"comparison":"AtLeast","value":parameter("minimum")}}, {"Logic":{"Not":{"Always":false}}}]}}}]);
    doc["skills"][0]["prepare_ms"] = json!(0);
    doc["skills"][0]["body"] = json!({"Sequence":{"id":"root","children":[
        {"Action":{"id":"hit","target":{"Literal":{"Target":"SelfUnit"}},"delivery":{"Literal":{"Delivery":"Instant"}},
            "effects":[{"Heal":{"id":"heal","tags":[],"amount":{"Literal":{"Amount":{"moment":"Live","root":{"Const":{"value":10,"unit":"Magnitude"}}}}}}}],"wait":"EffectsSettled"}},
        {"Branch":{"id":"decision","condition":{"Predicate":{"name":"positive-result","arguments":[
            {"name":"result","value":literal(json!({"FlowResult":{"node_id":"hit","effect_id":"heal","metric":"HpLost"}}))},
            {"name":"minimum","value":literal(json!({"FlowLiteral":{"Hp":1}}))}]}},
            "then_node":{"Wait":{"id":"yes","duration":{"Literal":{"Milliseconds":0}}}},"else_node":{"Wait":{"id":"no","duration":{"Literal":{"Milliseconds":0}}}}}}
    ]}});
    doc
}
#[test]
fn predicates_inline_with_typed_arguments_and_portable_origins() {
    let artifact = compile(&flow_module()).unwrap();
    assert_eq!(artifact.version, 2);
    assert!(!artifact.native_ron.contains("Predicate"));
    assert!(!artifact.native_ron.contains("Parameter"));
    let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    loaded.load_native().unwrap();
    let origins: Vec<_> = loaded
        .sources()
        .values()
        .filter(|s| s.definition.starts_with("predicates["))
        .collect();
    assert!(!origins.is_empty());
    assert!(origins
        .iter()
        .any(|s| s.calls.iter().any(|p| p.ends_with("children[1].condition"))));
    assert!(origins
        .iter()
        .any(|s| s.inputs.iter().any(|p| p.ends_with("arguments[0].value"))));
}
#[test]
fn predicates_preserve_v1_and_gate_new_features() {
    let old = compile_json(super::tests::FIXTURE).unwrap();
    assert_eq!(old.version, 1);
    Artifact::from_ron(&old.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let mut doc = flow_module();
    doc["version"] = json!(1);
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Version);
    doc["predicates"] = json!([]);
    doc["skills"][0]["body"]["Sequence"]["children"][1]["Branch"]["condition"] =
        json!({"Logic":{"Not":{"Always":false}}});
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Version);
}
#[test]
fn predicates_reject_wrong_units_missing_inputs_and_unresolved_values() {
    for (value, code) in [
        (
            literal(json!({"FlowLiteral":{"Count":1}})),
            DiagnosticCode::Type,
        ),
        (parameter("outside"), DiagnosticCode::Reference),
        (
            json!({"Unresolved":"threshold undecided"}),
            DiagnosticCode::Unresolved,
        ),
    ] {
        let mut doc = flow_module();
        call(&mut doc)["arguments"][1]["value"] = value;
        let error = compile(&doc).unwrap_err();
        assert_eq!(error.code, code, "{error:?}");
        assert!(error.source.definition.ends_with("arguments[1].value"));
    }
    let mut doc = flow_module();
    call(&mut doc)["arguments"].as_array_mut().unwrap().pop();
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Reference);
}
#[test]
fn predicates_check_unused_definitions_cycles_empty_groups_and_short_circuited_inputs() {
    let mut doc = flow_module();
    doc["predicates"].as_array_mut().unwrap().push(json!({"name":"unused","context":"Flow","parameters":[],"body":{"Logic":{"All":[{"Always":false},{"Call":{"name":"absent","arguments":[]}}]}}}));
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Reference);
    doc["predicates"][1]["body"] = json!({"Call":{"name":"unused","arguments":[]}});
    assert!(compile(&doc).unwrap_err().reason.contains("recursive"));
    doc["predicates"][1]["body"] = json!({"Logic":{"Any":[]}});
    assert!(compile(&doc).unwrap_err().reason.contains("nonempty"));
    doc["predicates"][1]["body"] = json!({"Result":{"result":literal(json!({"FlowResult":{"node_id":"hit","metric":"HpLost"}})),"comparison":"Equal","value":literal(json!({"FlowLiteral":{"Count":1}}))}});
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Type);
}
#[test]
fn predicates_validate_native_scope_inside_nested_boolean_nodes() {
    let mut doc = flow_module();
    call(&mut doc)["arguments"][0]["value"]["Literal"]["FlowResult"]["node_id"] = json!("missing");
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Reference);
    assert!(
        error.source.definition.starts_with("predicates[0].body"),
        "{error:?}"
    );
    let mut doc = flow_module();
    doc["skills"][0]["body"]["Sequence"]["children"][0]["Action"]["wait"] = json!("Released");
    assert!(compile(&doc)
        .unwrap_err()
        .reason
        .contains("not EffectsSettled"));
}
fn state_module() -> Json {
    let mut doc = parse_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/equipment_state.skill.yaml"
    ))
    .unwrap();
    doc["version"] = json!(2);
    doc["predicates"] = json!([{"name":"within-window","context":"State","parameters":[
        {"name":"now","value_type":{"Amount":"Milliseconds"}},{"name":"until","value_type":{"Amount":"Milliseconds"}},{"name":"armed","value_type":"BoolField"}],
        "body":{"Logic":{"All":[{"Compare":{"left":parameter("now"),"comparison":"Less","right":parameter("until")}}, {"Bool":{"field":parameter("armed"),"equals":true}}]}}}]);
    doc["values"][0]["value"]["StateProgram"]["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"armed","value":{"Bool":true}}));
    doc["values"][0]["value"]["StateProgram"]["handlers"][0]["body"][0]["If"]["condition"] = json!({"Predicate":{"name":"within-window","arguments":[
        {"name":"now","value":literal(amount(0,"Milliseconds"))},{"name":"until","value":literal(amount(5000,"Milliseconds"))},{"name":"armed","value":literal(json!({"BoolField":"armed"}))}]}});
    doc
}
#[test]
fn predicates_lower_equipped_state_and_check_actual_field_context() {
    let doc = state_module();
    let artifact = compile(&doc).unwrap();
    let saved = artifact.to_ron().unwrap();
    assert!(!saved.contains("Predicate("));
    Artifact::from_ron(&saved).unwrap().load_native().unwrap();
    let mut bad = doc.clone();
    bad["values"][0]["value"]["StateProgram"]["handlers"][0]["body"][0]["If"]["condition"]
        ["Predicate"]["arguments"][2]["value"] = literal(json!({"BoolField":"last_stack"}));
    assert!(compile(&bad).unwrap_err().reason.contains("Bool field"));
    let mut bad = doc;
    bad["predicates"][0]["context"] = json!("Flow");
    assert_eq!(compile(&bad).unwrap_err().code, DiagnosticCode::Type);
}
#[test]
fn predicates_bound_total_inline_growth_and_unused_dag_depth() {
    let mut doc = flow_module();
    doc["predicates"] =
        json!([{"name":"leaf","context":"Flow","parameters":[],"body":{"Always":true}}]);
    for i in 1..14 {
        let target = if i == 1 {
            "leaf".into()
        } else {
            format!("p{}", i - 1)
        };
        doc["predicates"].as_array_mut().unwrap().push(json!({"name":format!("p{i}"),"context":"Flow","parameters":[],"body":{"Logic":{"All":[{"Call":{"name":target,"arguments":[]}},{"Call":{"name":target,"arguments":[]}}]}}}));
    }
    *call(&mut doc) = json!({"name":"p13","arguments":[]});
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Limit);
    let mut doc = flow_module();
    for i in 0..34 {
        let body = if i == 0 {
            json!({"Always":true})
        } else {
            json!({"Call":{"name":format!("unused{}",i-1),"arguments":[]}})
        };
        doc["predicates"].as_array_mut().unwrap().push(
            json!({"name":format!("unused{i}"),"context":"Flow","parameters":[],"body":body}),
        );
    }
    assert_eq!(compile(&doc).unwrap_err().code, DiagnosticCode::Limit);
}

#[test]
fn predicates_ranges_reject_unproven_and_out_of_range_arguments() {
    let mut doc = flow_module();
    doc["predicates"][0]["parameters"][1]["range"] = json!({"min":1,"max":10});
    compile(&doc).unwrap();
    call(&mut doc)["arguments"][1]["value"] = literal(json!({"FlowLiteral":{"Hp":11}}));
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Type);
    assert!(error.reason.contains("range 1..10"));
    let mut doc = state_module();
    doc["predicates"][0]["parameters"][0]["range"] = json!({"min":0,"max":5000});
    doc["values"][0]["value"]["StateProgram"]["handlers"][0]["body"][0]["If"]["condition"]
        ["Predicate"]["arguments"][0]["value"] =
        literal(json!({"Amount":{"moment":"Live","root":{"Read":{"Time":"BattleElapsed"}}}}));
    assert!(compile(&doc)
        .unwrap_err()
        .reason
        .contains("must prove range"));
}

#[test]
fn predicates_native_compilers_reject_unexpanded_calls_and_legacy_operation_limits_survive() {
    use crate::game::ability::{flow::*, state_program::*};
    let raw: Module = serde_json::from_value(state_module()).unwrap();
    let Value::StateProgram(program) = &raw.values[0].value else {
        panic!()
    };
    assert!(program
        .compile_equipped(&crate::game::resource::ResourceCatalog { resources: vec![] }, None, &crate::game::value_expr::ExprLimits::default())
        .unwrap_err()
        .contains("unresolved authored predicate"));
    let condition: FlowCondition =
        serde_json::from_value(json!({"Predicate":{"name":"absent","arguments":[]}})).unwrap();
    let flow = std::sync::Arc::new(SkillFlowDef::Branch {
        id: "b".into(),
        condition,
        then_node: Box::new(SkillFlowDef::Wait {
            id: "yes".into(),
            duration_ms: 0,
        }),
        else_node: Box::new(SkillFlowDef::Wait {
            id: "no".into(),
            duration_ms: 0,
        }),
    });
    assert!(CompiledSkillFlow::compile(Some(&flow), &[])
        .unwrap_err()
        .cause
        .contains("unresolved authored predicate"));
    let mut doc = state_module();
    doc["version"] = json!(1);
    doc["predicates"] = json!([]);
    doc["values"][0]["value"]["StateProgram"]["handlers"][0]["body"] = json!(vec![
        json!({"If":{"condition":{"Bool":{"field":"armed","equals":true}},"then_ops":[],"else_ops":[]}});
        256
    ]);
    let value = doc["values"][0].clone();
    for i in 0..17 {
        let mut copy = value.clone();
        copy["name"] = json!(format!("unused-{i}"));
        doc["values"].as_array_mut().unwrap().push(copy);
    }
    compile(&doc)
        .expect("existing per-program 256-op allowance is not a new aggregate module allowance");
}

#[test]
fn predicates_unused_effects_check_actual_state_fields_and_events() {
    let mut doc = state_module();
    let mut program = doc["values"][0]["value"]["StateProgram"].clone();
    program["handlers"][0]["body"][0]["If"]["condition"]["Predicate"]["arguments"][2]["value"] =
        literal(json!({"BoolField":"missing"}));
    doc["values"].as_array_mut().unwrap().push(json!({"name":"unused-state","value":{"Effects":[{"id":"state","op":{"Sustained":{"definition":{"State":{"duration_ms":1000,"program":program}}}}}]}}));
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert!(error.reason.contains("unknown field"), "{error:?}");
    let mut doc = state_module();
    let mut program = doc["values"][0]["value"]["StateProgram"].clone();
    program["handlers"][0]["event"] = json!("Expired");
    program["handlers"][0]["body"] = json!([{"If":{"condition":{"Logic":{"All":[{"Bool":{"field":"armed","equals":false}},{"Compare":{"left":{"moment":"Live","root":{"Read":"EventHpLost"}},"comparison":"Greater","right":{"moment":"Live","root":{"Const":{"value":0,"unit":"Magnitude"}}}}}]}},"then_ops":[],"else_ops":[]}}]);
    doc["values"].as_array_mut().unwrap().push(json!({"name":"unused-event","value":{"Effects":[{"id":"state","op":{"Sustained":{"definition":{"State":{"duration_ms":1000,"program":program}}}}}]}}));
    let error = compile(&doc).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert!(
        error.reason.contains("unavailable state event context"),
        "{error:?}"
    );
}
