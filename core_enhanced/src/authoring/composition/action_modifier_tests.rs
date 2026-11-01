use super::*;
use serde_json::json;

#[test]
fn composition_support_radius_preserves_legacy_pattern_wire_and_circular_source() {
    use crate::game::relic::structural::support::SupportPolicyDef;
    let old = r#"(range:(include_anchor_tile:true,rows:["@X"]))"#;
    let policy: SupportPolicyDef = ron::de::from_str(old).unwrap();
    assert!(policy.range.is_some());
    assert_eq!(ron::ser::to_string(&policy).unwrap(), r#"(range:(include_anchor_tile:true,rows:["@X",],),)"#);
    let json = serde_json::to_value(&policy).unwrap();
    assert_eq!(json["range"]["rows"], json!(["@X"]));
    let circle: SupportPolicyDef = ron::de::from_str("()").unwrap();
    assert!(circle.range.is_none());
    assert_eq!(ron::ser::to_string(&circle).unwrap(), "()");
    assert!(serde_json::from_value::<SupportPolicyDef>(json!({"range":null})).is_err());

    let source = include_str!("../../../tests/fixtures/skill_composition/support_radius.skill.yaml");
    let artifact = compile_yaml(source).unwrap();
    let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    let native = artifact.load_native().unwrap();
    let crate::game::ability::action::ActionChange::SupportRadius { step, effect, percent } =
        &native.skills[0].action_policy.as_ref().unwrap().modifiers[0].change else { panic!() };
    assert_eq!(step, &native.skills[0].steps[0].id);
    assert_eq!(effect, "support");
    assert_eq!(*percent, 25);
    let origin = &artifact.sources["skills[0].action_policy.modifiers[0].change.SupportRadius.percent"];
    assert_eq!(origin.definition, "values[1].value.Count");
    assert!(origin.inputs.iter().any(|p| p.ends_with("change.SupportRadius.percent")));
    for (value, code) in [(json!({"Milliseconds":25}), DiagnosticCode::Type), (json!({"Count":0}), DiagnosticCode::Native), (json!({"Count":2147483648u32}), DiagnosticCode::Native)] {
        let mut doc = parse_yaml(source).unwrap();
        doc["values"][1]["value"] = value;
        assert_eq!(compile_json(&doc.to_string()).unwrap_err().code, code);
    }
    for mask in [false, true] {
        let mut doc = parse_yaml(source).unwrap();
        let aura = &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["ChannelAura"]["aura"];
        if mask { aura["support"]["range"] = json!({"include_anchor_tile":true,"rows":["@X"]}); }
        else { aura["radius_units"] = json!(0); }
        let error = compile_json(&doc.to_string()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Native, "{error:?}");
        assert!(error.reason.contains(if mask { "circular" } else { "positive" }), "{error:?}");
    }
}

const YAML: &str =
    include_str!("../../../tests/fixtures/skill_composition/duration_modifiers.skill.yaml");

#[test]
fn composition_duration_modifiers_expand_each_call_and_keep_input_origins() {
    let mut doc = parse_yaml(YAML).unwrap();
    doc["skills"][0]["prepare_ms"] = json!(17);
    let first = doc["skills"][0]["body"].clone();
    let mut second = first.clone();
    second["Call"]["id"] = json!("another-support");
    second["Call"]["arguments"][1]["value"] = json!({"Literal":{"Count":50}});
    doc["skills"][0]["body"] = json!({"Sequence":{"id":"twice","children":[first,second]}});
    let mut other = doc["skills"][0].clone();
    other["id"] = json!("unmodified");
    other["body"] = doc["definitions"][0]["body"].clone();
    other["body"]["Action"]["modifiers"] = json!([]);
    other["body"]["Action"]["effects"][0]["Append"]["effects"] =
        json!({"Shared":"regular-support"});
    doc["skills"].as_array_mut().unwrap().push(other);
    let artifact = compile_json(&doc.to_string()).unwrap();
    let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    let native = artifact.load_native().unwrap();
    let skill = &native.skills[0];
    assert_eq!(skill.focus_time_ms, 17);
    let policy = skill.action_policy.as_ref().unwrap();
    assert_eq!(policy.modifiers.len(), 2);
    assert_ne!(policy.modifiers[0].id, policy.modifiers[1].id);
    assert!(native.skills[1].action_policy.is_none());
    for (i, expected) in [25, 50].into_iter().enumerate() {
        let crate::game::ability::action::ActionChange::Duration {
            step,
            effect,
            percent,
        } = &policy.modifiers[i].change
        else {
            panic!()
        };
        assert_eq!(step, &skill.steps[i].id);
        assert_eq!(effect, "support");
        assert_eq!(*percent, expected);
        let origin = &artifact.sources
            [&format!("skills[0].action_policy.modifiers[{i}].change.Duration.percent")];
        assert!(origin
            .calls
            .iter()
            .any(|p| p == &format!("skills[0].body.children[{i}]")));
        assert!(origin
            .inputs
            .iter()
            .any(|p| p == "definitions[0].body.modifiers[0].change.Duration.percent"));
        assert_eq!(
            origin.definition,
            if i == 0 {
                "values[1].value.Count".to_owned()
            } else {
                "skills[0].body.children[1].arguments[1].value.Literal.Count".to_owned()
            }
        );
    }
}

#[test]
fn composition_duration_modifiers_expansion_limit_rejects_before_native_publication() {
    let mut doc = parse_yaml(YAML).unwrap();
    let call = doc["skills"][0]["body"].clone();
    let children: Vec<_> = (0..129)
        .map(|i| {
            let mut next = call.clone();
            next["Call"]["id"] = json!(format!("call-{i}"));
            next
        })
        .collect();
    doc["skills"][0]["body"] = json!({"Sequence":{"id":"many","children":children}});
    let error = compile_json(&doc.to_string()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Limit);
    assert!(error.reason.contains("expanded skill action modifier"));
    assert!(error
        .source
        .calls
        .iter()
        .any(|p| p == "skills[0].body.children[128]"));
    compile_yaml(YAML).unwrap();
}

#[test]
fn composition_duration_modifiers_reject_wrong_units_values_and_effects_at_source() {
    let doc = parse_yaml(YAML).unwrap();
    for (value, code) in [
        (json!({"Milliseconds":25}), DiagnosticCode::Type),
        (json!({"Count":0}), DiagnosticCode::Native),
        (json!({"Count":2147483648u32}), DiagnosticCode::Native),
    ] {
        let mut invalid = doc.clone();
        invalid["values"][1]["value"] = value;
        let error = compile_json(&invalid.to_string()).unwrap_err();
        assert_eq!(error.code, code, "{error:?}");
        assert!(!error.source.calls.is_empty(), "{error:?}");
        if error.code == DiagnosticCode::Native {
            assert_eq!(error.source.definition, "values[1].value.Count");
        }
    }
    let mut invalid = doc.clone();
    invalid["definitions"][0]["body"]["Action"]["modifiers"][0]["change"]["Duration"]["effect"] =
        json!("missing");
    let error = compile_json(&invalid.to_string()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Reference);
    assert_eq!(
        error.source.definition,
        "definitions[0].body.modifiers[0].change.Duration.effect"
    );
    let mut invalid = doc.clone();
    invalid["values"][0]["value"]["Effects"] = json!([]);
    assert_eq!(
        compile_json(&invalid.to_string()).unwrap_err().code,
        DiagnosticCode::Reference
    );
    let mut invalid = doc.clone();
    invalid["definitions"][0]["body"]["Action"]["effects"] = json!([{"Heal":{"id":"support","tags":[],"amount":{"Literal":{"Amount":{"moment":"Snapshot","root":{"Const":{"value":1,"unit":"Magnitude"}}}}}}}]);
    assert!(compile_json(&invalid.to_string())
        .unwrap_err()
        .reason
        .contains("finite sustained"));
    let mut invalid = doc.clone();
    let modifier = invalid["definitions"][0]["body"]["Action"]["modifiers"][0].clone();
    invalid["definitions"][0]["body"]["Action"]["modifiers"]
        .as_array_mut()
        .unwrap()
        .push(modifier);
    assert!(compile_json(&invalid.to_string())
        .unwrap_err()
        .reason
        .contains("duplicate local"));
    // Literal invalid input cannot hide in a definition that is never called.
    let mut invalid = doc;
    invalid["skills"][0]["body"] =
        json!({"Wait":{"id":"unused","duration":{"Literal":{"Milliseconds":1}}}});
    invalid["definitions"][0]["body"]["Action"]["modifiers"][0]["change"]["Duration"]["percent"] =
        json!({"Literal":{"Count":0}});
    assert!(compile_json(&invalid.to_string())
        .unwrap_err()
        .source
        .definition
        .ends_with("percent.Literal.Count"));
}

const CHANNEL_MODIFIERS: &str = include_str!("../../../tests/fixtures/skill_composition/channel_modifiers.skill.yaml");

#[test]
fn composition_channel_modifiers_scope_and_input_sources_survive_roundtrip() {
    use crate::game::ability::action::{ActionChange as Native, AreaChange};
    let mut doc = parse_yaml(CHANNEL_MODIFIERS).unwrap();
    let mut other = doc["skills"][0].clone();
    other["id"] = json!("unprotected");
    other["modifiers"] = json!([]);
    doc["skills"].as_array_mut().unwrap().push(other);
    let artifact = compile_json(&doc.to_string()).unwrap();
    let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    let native = artifact.load_native().unwrap();
    let protected = native.skills[0].action_policy.as_ref().unwrap();
    assert!(!protected.external_interrupt_protected); // Requires an actual equipped copy.
    assert_eq!(protected.modifiers.len(), 4);
    assert_eq!(protected.modifiers[0].change, Native::ProtectInterruption);
    assert!(native.skills[1].action_policy.as_ref().unwrap().modifiers.iter().all(|m| m.change != Native::ProtectInterruption));
    let Native::Area { step, change: AreaChange::RampRate { status, reached_rules, numerator, denominator } } = &protected.modifiers[3].change else { panic!() };
    assert_eq!(step, &native.skills[0].steps[0].id);
    assert_eq!((*status, reached_rules.as_slice(), *numerator, *denominator), (0, &[2][..], 3, 2));
    assert_eq!(artifact.sources["skills[0].action_policy.modifiers[0].change"].definition, "skills[0].modifiers[0].change");
    assert_eq!(artifact.sources["skills[0].action_policy.modifiers[3].change.Area.change.RampRate.numerator"].definition, "values[3].value.Count");
    for p in ["values[3].value", "values[4].value", "skills[0].modifiers[0]"] {
        assert!(artifact.sources["skills[0].steps[0].delivery"].inputs.iter().any(|v| v == p));
    }
}

#[test]
fn composition_channel_modifiers_reject_invalid_scope_units_ratio_and_references() {
    let doc = parse_yaml(CHANNEL_MODIFIERS).unwrap();
    for (field, value) in [("status", json!(9)), ("status", json!(1)), ("reached_rules", json!([0])), ("reached_rules", json!([2,2]))] {
        let mut invalid = doc.clone();
        invalid["definitions"][0]["body"]["Action"]["modifiers"][2]["change"]["AreaRampRate"][field] = value;
        let e = compile_json(&invalid.to_string()).unwrap_err();
        assert_eq!(e.code, DiagnosticCode::Native, "{e:?}");
        assert!(e.source.definition.contains("AreaRampRate"), "{e:?}");
    }
    for unused in [false, true] {
        for (index, value) in [(3, json!({"Count":0})), (4, json!({"Count":0})), (4, json!({"Count":4})), (3, json!({"Milliseconds":3}))] {
            let mut invalid = doc.clone();
            invalid["values"][index]["value"] = value;
            if unused { invalid["skills"][0]["modifiers"] = json!([]); invalid["skills"][0]["body"] = json!({"Wait":{"id":"idle","duration":{"Literal":{"Milliseconds":1}}}}); }
            let error = compile_json(&invalid.to_string()).unwrap_err();
            assert!(matches!(error.code, DiagnosticCode::Native | DiagnosticCode::Type), "{error:?}");
        }
    }
    let mut invalid = doc.clone();
    invalid["definitions"][0]["body"]["Action"]["modifiers"][0]["change"] = json!("ProtectInterruption");
    assert_eq!(compile_json(&invalid.to_string()).unwrap_err().code, DiagnosticCode::Parse);
    let mut invalid = doc.clone();
    invalid["skills"][0]["modifiers"][0]["change"] = json!({"AreaDuration":{"percent":{"Literal":{"Count":25}}}});
    assert_eq!(compile_json(&invalid.to_string()).unwrap_err().code, DiagnosticCode::Parse);
    let mut invalid = doc;
    let modifier = invalid["skills"][0]["modifiers"][0].clone();
    invalid["skills"][0]["modifiers"].as_array_mut().unwrap().push(modifier);
    assert!(compile_json(&invalid.to_string()).unwrap_err().reason.contains("duplicate skill"));
}

#[test]
fn composition_channel_modifiers_check_parameter_ratios_per_call_and_combined_limit() {
    let mut doc = parse_yaml(CHANNEL_MODIFIERS).unwrap();
    doc["definitions"][0]["parameters"].as_array_mut().unwrap().push(json!({"name":"speed","value_type":"Count"}));
    doc["definitions"][0]["body"]["Action"]["modifiers"][2]["change"]["AreaRampRate"]["numerator"] = json!({"Parameter":"speed"});
    doc["skills"][0]["body"]["Call"]["arguments"].as_array_mut().unwrap().push(json!({"name":"speed","value":{"Literal":{"Count":1}}}));
    assert!(compile_json(&doc.to_string()).unwrap_err().reason.contains("ratio"));
    doc["skills"][0]["body"]["Call"]["arguments"][1]["value"] = json!({"Literal":{"Count":3}});
    compile_json(&doc.to_string()).unwrap();
    let m = doc["skills"][0]["modifiers"][0].clone();
    doc["skills"][0]["modifiers"] = json!((0..128).map(|i| { let mut m = m.clone(); m["id"] = json!(format!("protection-{i}")); m }).collect::<Vec<_>>());
    let e = compile_json(&doc.to_string()).unwrap_err();
    assert_eq!(e.code, DiagnosticCode::Limit);
    assert!(e.reason.contains("expanded skill"));
}

#[test]
fn composition_channel_modifiers_reject_whole_skill_condition_without_action() {
    let mut doc = parse_yaml(CHANNEL_MODIFIERS).unwrap();
    doc["skills"][0]["body"] = json!({"Wait":{"id":"idle","duration":{"Literal":{"Milliseconds":1}}}});
    let error = compile_json(&doc.to_string()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert_eq!(error.source.definition, "skills[0].modifiers");
    assert!(error.reason.contains("authored Action"));
}

const VULNERABILITY: &str = include_str!("../../../tests/fixtures/skill_composition/control_vulnerability.skill.yaml");
const FREEZE_VULNERABILITY: &str = include_str!("../../../tests/fixtures/skill_composition/freeze_vulnerability.skill.yaml");
#[test]
fn composition_control_vulnerability_roundtrip_shared_parameters_and_origins() {
    for (source, index, path, percent_index, duration_index, native_path) in [
        (VULNERABILITY, 4, "skills[0].steps[0].delivery", 5, 6, "Area.change.ControlVulnerability"),
        (FREEZE_VULNERABILITY, 0, "skills[0].steps[0].effects[0]", 1, 2, "ControlVulnerability"),
    ] {
        let a = compile_yaml(source).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        let n = a.load_native().unwrap();
        assert!(n.skills[0].action_policy.is_some());
        let p = format!("skills[0].action_policy.modifiers[{index}].change.{native_path}");
        assert_eq!(a.sources[&format!("{p}.percent")].definition, format!("values[{percent_index}].value.Count"));
        assert_eq!(a.sources[&format!("{p}.duration_ms")].definition, format!("values[{duration_index}].value.Milliseconds"));
        for i in [percent_index, duration_index] {
            assert!(a.sources[path].inputs.contains(&format!("values[{i}].value")));
        }
    }
}
#[test]
fn composition_control_vulnerability_rejects_bad_units_bounds_and_control_references() {
    for (source, percent, duration) in [(VULNERABILITY, 5, 6), (FREEZE_VULNERABILITY, 1, 2)] {
        let d = parse_yaml(source).unwrap();
        for (i, value) in [(percent,json!({"Milliseconds":15})),(percent,json!({"Count":0})),(percent,json!({"Count":2147483648u32})),(duration,json!({"Count":5000})),(duration,json!({"Milliseconds":0})),(duration,json!({"Milliseconds":4294967296u64}))] {
            let mut bad = d.clone();
            bad["values"][i]["value"] = value;
            let e = compile_json(&bad.to_string()).unwrap_err();
            assert!(matches!(e.code,DiagnosticCode::Type|DiagnosticCode::Native), "{e:?}");
            assert!(e.source.definition.contains("values[") || e.source.definition.contains("ControlVulnerability") || (e.code == DiagnosticCode::Type && e.source.definition.starts_with("skills[0].body.arguments[")), "{e:?}");
        }
    }
    for status in [0,2,99] {
        let mut bad = parse_yaml(VULNERABILITY).unwrap();
        bad["definitions"][0]["body"]["Action"]["modifiers"][3]["change"]["AreaControlVulnerability"]["status"] = json!(status);
        let e = compile_json(&bad.to_string()).unwrap_err();
        assert!(e.reason.contains("stun"), "{e:?}");
    }
    for kind in [json!("Root"),json!({"Slow":{"percent":20}}),json!({"DamageTaken":{"percent":20}})] {
        let mut bad = parse_yaml(FREEZE_VULNERABILITY).unwrap();
        bad["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["HitStatus"]["profile"]["kind"] = kind;
        let e = compile_json(&bad.to_string()).unwrap_err();
        assert!(e.reason.contains("stun or freeze"), "{e:?}");
    }
    let mut bad = parse_yaml(FREEZE_VULNERABILITY).unwrap();
    bad["definitions"][0]["body"]["Action"]["modifiers"][0]["change"]["ControlVulnerability"]["effect"] = json!("missing");
    assert_eq!(compile_json(&bad.to_string()).unwrap_err().code, DiagnosticCode::Reference);
}
#[test]
fn composition_control_vulnerability_checks_unused_concrete_input_and_each_call() {
    let mut d = parse_yaml(VULNERABILITY).unwrap();
    d["skills"][0]["modifiers"] = json!([]);
    d["skills"][0]["body"] = json!({"Wait":{"id":"idle","duration":{"Literal":{"Milliseconds":1}}}});
    d["values"][6]["value"] = json!({"Milliseconds":0});
    let e = compile_json(&d.to_string()).unwrap_err();
    assert_eq!(e.source.definition, "values[6].value.Milliseconds");
    let mut d = parse_yaml(FREEZE_VULNERABILITY).unwrap();
    let first = d["skills"][0]["body"].clone();
    let mut second = first.clone();
    second["Call"]["id"] = json!("second");
    second["Call"]["arguments"][2]["value"] = json!({"Literal":{"Milliseconds":0}});
    d["skills"][0]["body"] = json!({"Sequence":{"id":"twice","children":[first,second]}});
    let e = compile_json(&d.to_string()).unwrap_err();
    assert!(e.source.definition.contains("children[1].arguments[2]"), "{e:?}");
    d["skills"][0]["body"]["Sequence"]["children"][1]["Call"]["arguments"][2]["value"] = json!({"Literal":{"Milliseconds":3000}});
    let a = compile_json(&d.to_string()).unwrap();
    assert_eq!(a.load_native().unwrap().skills[0].action_policy.as_ref().unwrap().modifiers.len(), 2);
}

#[test]
fn composition_control_vulnerability_native_policy_rejects_noncontrol_and_existing_writer() {
    use crate::game::ability::{SkillEffectDef, sustained::{SustainedEffectDef, HitStatusKind, HitVulnerability}};
    let a = compile_yaml(FREEZE_VULNERABILITY).unwrap();
    for existing in [false, true] {
        let mut native = a.load_native().unwrap();
        let skill = &mut native.skills[0];
        let SkillEffectDef::Sustained { definition: SustainedEffectDef::HitStatus { profile, .. } } = &mut skill.steps[0].effects[0].op else { panic!() };
        if existing { profile.vulnerability = Some(HitVulnerability { definition:"already".into(),percent:15,duration_ms:5000,modifier_source:None }); }
        else { profile.kind = HitStatusKind::Root; }
        assert!(skill.action_policy.as_ref().unwrap().validate(skill).unwrap_err().contains("stun or freeze"));
    }
}
