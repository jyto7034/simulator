use super::*;
use serde_json::{json, Value};

pub(crate) fn scheduled_area_fixture() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../content_assembler/content/rudolta.node-graph.json"),
    )
    .unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-count-positive" => node["values"]["value"] = json!(3),
            "skill-seconds" => {
                node["values"]["value"] = if node["title"].as_str().unwrap().contains("간격") {
                    json!(0.1)
                } else {
                    json!(0.2)
                }
            }
            "skill-range" => {
                node["values"]["pattern"] = if node["title"].as_str().unwrap().contains("착탄") {
                    json!("XXX\nX@X\nXXX")
                } else {
                    json!("XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX")
                };
                node["values"]["includeAnchor"] = json!(true);
            }
            "skill-weapon-power" => node["values"]["coefficient"] = json!(1.5),
            _ => {}
        }
    }
    doc
}
#[test]
fn scheduled_area_graph_preserves_finite_opportunities_and_child_delay() {
    let graph = compile(&scheduled_area_fixture().to_string(), "scheduled".into()).unwrap();
    let ron = graph.to_ron().unwrap();
    let restored = PortableGraph::from_ron(&ron).unwrap();
    assert_eq!(graph.operation(), restored.operation());
    let GraphOperation::ResonanceScheduledArea { program, .. } = graph.operation() else {
        panic!("wrong operation");
    };
    assert_eq!(
        (
            program.opportunities,
            program.interval_ms,
            program.duration_ms().unwrap()
        ),
        (3, 100, 201)
    );
    assert_eq!(program.impact.delivery_delay_ms, 200);
    assert_eq!(program.impact.acquisition.rows.len(), 5);
    assert_eq!(program.impact.footprint.rows.len(), 3);
    restored.load_native().unwrap();
    assert!(PortableGraph::from_ron(&ron.replace("opportunities: 3", "opportunities: 0")).is_err());
    assert!(
        PortableGraph::from_ron(&ron.replace("opportunities: 3", "opportunities: 10001")).is_err()
    );
}

pub(crate) fn delayed_area_fixture() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../content_assembler/content/clouded-monk.node-graph.json"),
    )
    .unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-seconds" => {
                node["values"]["value"] = if node["title"].as_str().unwrap().contains("준비") {
                    json!(0.1)
                } else {
                    json!(0.2)
                }
            }
            "skill-range" => {
                node["values"]["pattern"] = if node["title"].as_str().unwrap().contains("착탄") {
                    json!("XXX\nX@X\nXXX")
                } else {
                    json!("XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX")
                };
                node["values"]["includeAnchor"] = json!(true);
            }
            "skill-weapon-power" => node["values"]["coefficient"] = json!(1.5),
            "skill-self-heal-plan" => node["values"]["percent"] = json!(20),
            _ => {}
        }
    }
    doc
}

#[test]
fn delayed_area_graph_preserves_preparation_independent_delay_power_and_heal() {
    let graph = compile(&delayed_area_fixture().to_string(), "delayed".into()).unwrap();
    let ron = graph.to_ron().unwrap();
    let restored = PortableGraph::from_ron(&ron).unwrap();
    assert_eq!(graph.operation(), restored.operation());
    let GraphOperation::ResonanceDelayedArea { program, .. } = graph.operation() else {
        panic!("wrong operation");
    };
    assert_eq!((program.prepare_ms, program.delivery_delay_ms), (100, 200));
    assert_eq!(
        (
            program.coefficient_numerator,
            program.coefficient_denominator
        ),
        (3, 2)
    );
    assert_eq!(program.on_hit_heal_percent, Some(20));
    restored.load_native().unwrap();
    assert!(PortableGraph::from_ron(
        &ron.replace("coefficient_denominator: 2", "coefficient_denominator: 0")
    )
    .is_err());
}

pub(crate) fn reactive_heal_fixture() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../content_assembler/content/queen-bee.node-graph.json"),
    )
    .unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-seconds" => node["values"]["value"] = json!(2),
            "skill-range" => {
                node["values"]["pattern"] =
                    json!("XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX");
                node["values"]["includeAnchor"] = json!(true);
            }
            "skill-reactive-heal-plan" => {
                node["values"]["charges"] = json!(2);
                node["values"]["percent"] = json!(20);
            }
            _ => {}
        }
    }
    doc
}

#[test]
fn reactive_heal_graph_preserves_recipient_range_charges_and_duration_in_ir() {
    let graph = compile(&reactive_heal_fixture().to_string(), "reactive".into()).unwrap();
    let restored = PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();
    assert_eq!(graph.operation(), restored.operation());
    let GraphOperation::ResonanceReactiveHeal {
        profile,
        duration_ms,
        range,
        ..
    } = graph.operation()
    else {
        panic!("wrong operation");
    };
    assert_eq!(
        (profile.charges, profile.max_health_percent, *duration_ms),
        (2, 20, 2000)
    );
    assert_eq!(range.rows.len(), 7);
    assert!(range.include_anchor_tile);
    assert_eq!(
        restored
            .load_native()
            .unwrap()
            .get_by_id("reactive")
            .unwrap()
            .cast_selection(),
        crate::game::ability::SkillCastSelection::PlayerUnit
    );
}

/// Values only for execution tests; the official graph retains open balance fields.
pub(crate) fn state_fixture() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../content_assembler/content/schadenfreude.node-graph.json"),
    )
    .unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-seconds" => node["values"]["value"] = json!(1),
            "skill-attribute-effect" => node["values"]["amount"] = json!(50),
            _ => {}
        }
    }
    doc
}

#[test]
fn target_state_graph_compiles_real_connections_and_revalidates_transport() {
    let original = state_fixture();
    let artifact = compile(&original.to_string(), "generic_state".into()).unwrap();
    let transported = PortableGraph::from_ron(&artifact.to_ron().unwrap()).unwrap();
    assert_eq!(transported.operation(), artifact.operation());
    let GraphOperation::ResonanceState {
        duration_ms,
        profile,
        ..
    } = transported.operation()
    else {
        panic!("wrong operation");
    };
    assert_eq!(*duration_ms, 1000);
    assert_eq!(
        (
            profile.attack_percent,
            profile.defense_percent,
            profile.attack_speed_points
        ),
        (50, 50, 50)
    );
    assert!(profile.hp_floor_one && profile.end_on_source_exit);
    let db = transported.load_native().unwrap();
    assert_eq!(
        db.get_by_id("generic_state").unwrap().steps[0]
            .effects
            .len(),
        1
    );
    for (kind, field, value) in [
        ("skill-attribute-effect", "amount", json!(10001)),
        ("skill-attribute-effect", "amount", json!(0.5)),
        ("skill-hp-floor-effect", "minimum", json!(2)),
        ("skill-action-entry", "start", json!("next-basic")),
        ("skill-recipient-buff-plan", "recipient", json!("selected")),
    ] {
        let mut invalid = original.clone();
        let node = invalid["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["kind"] == kind)
            .unwrap();
        node["values"][field] = value;
        assert!(
            compile(&invalid.to_string(), "generic_state".into()).is_err(),
            "{kind}.{field}"
        );
    }
    let mut corrupt = transported;
    let GraphOperation::ResonanceState { profile, .. } = &mut corrupt.operation else {
        unreachable!()
    };
    profile.attack_speed_points = u32::MAX;
    assert!(corrupt.load_native().is_err());
}

fn fixture(id: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../content_assembler/docs/examples/compiler-preflight")
            .join(format!("{id}.node-graph.json")),
    )
    .unwrap()
}
#[test]
fn normalization_preserves_reroute_origins_and_ignores_presentation_order() {
    let original = read_and_normalize(&fixture("G01")).unwrap();
    let routed = read_and_normalize(&fixture("G14")).unwrap();
    let moved = read_and_normalize(&fixture("G15")).unwrap();
    assert_eq!(original.semantic_fingerprint, routed.semantic_fingerprint);
    assert_eq!(original.semantic_fingerprint, moved.semantic_fingerprint);
    assert_ne!(original.digest, routed.digest);
    assert_ne!(original.digest, moved.digest);
    let origin = &routed.nodes["admit"].inputs["permit"][0];
    assert_eq!(origin.source.node, "qualify");
    assert_eq!(origin.path.len(), 2);
    assert_eq!(origin.reroutes.len(), 1);
    // Portable authoring normalization can round-trip; this is not an executable IR receipt.
    let ron = ron::ser::to_string(&routed).unwrap();
    let loaded: NormalizedDocument = ron::de::from_str(&ron).unwrap();
    assert_eq!(routed, loaded);
}
#[test]
fn malformed_supported_authoring_inputs_retain_the_original_location() {
    for (case, code, node) in [
        ("G04", "graph.port_type", Some("settings")),
        ("G05", "graph.unknown_kind", Some("cost")),
        ("G06", "graph.reference", None),
        ("G10", "graph.cycle", Some("left")),
        ("G13", "graph.integer_required", Some("amount")),
        ("G16", "graph.version", None),
    ] {
        let errors = read_and_normalize(&fixture(case)).unwrap_err();
        let error = errors
            .iter()
            .find(|e| e.code == code)
            .unwrap_or_else(|| panic!("{case}: {errors:?}"));
        if let Some(node) = node {
            assert_eq!(error.primary.node.as_deref(), Some(node));
        }
        assert_eq!(error.digest.len(), 64);
        if case == "G04" {
            assert_eq!(error.primary.port.as_deref(), Some("capacity"));
            assert_eq!(error.related[0].node.as_deref(), Some("seconds"));
            assert!(error.primary.edge.is_some());
        }
    }
}
#[test]
fn all_current_official_graphs_can_be_read_without_filling_balance_gaps() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content");
    let mut count = 0;
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if !path.to_string_lossy().ends_with(".node-graph.json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let result =
            read_and_normalize(&text).unwrap_or_else(|e| panic!("{}: {e:?}", path.display()));
        assert!(!result.nodes.is_empty());
        count += 1;
    }
    assert!(count >= 20);
    let incomplete = read_and_normalize(&fixture("G03")).unwrap();
    assert_eq!(
        incomplete.nodes["capacity"].values["value"],
        FieldValue::Text(String::new())
    );
    let gaps = authoring_gaps(&incomplete);
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].code, "graph.value_unresolved");
    assert_eq!(gaps[0].primary.field.as_deref(), Some("value"));
    assert!(authoring_gaps(&read_and_normalize(&fixture("G01")).unwrap()).is_empty());
}
#[test]
fn inactive_fields_are_preserved_in_source_but_do_not_change_normalized_meaning() {
    let mut original: Value = serde_json::from_str(&fixture("G01")).unwrap();
    let before = read_and_normalize(&original.to_string()).unwrap();
    let cost = original["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["id"] == "cost")
        .unwrap();
    assert_eq!(cost["values"]["mode"], "all");
    cost["values"]["percent"] = json!(37);
    let after = read_and_normalize(&original.to_string()).unwrap();
    assert_eq!(before.semantic_fingerprint, after.semantic_fingerprint);
    assert_ne!(before.digest, after.digest);
}
#[test]
fn time_precision_uses_decimal_milliseconds_and_never_rounds_invalid_values() {
    for (seconds, expected) in [
        (0.0, Some(0)),
        (1.001, Some(1001)),
        (0.001, Some(1)),
        (0.0001, None),
        (-1.0, None),
        (9_007_199_254_741.0, None),
    ] {
        assert_eq!(super::structure::milliseconds(&json!(seconds)), expected);
    }
}

#[test]
fn request_payment_and_receipt_scopes_are_checked_after_reroute_normalization() {
    for case in ["G01", "G02", "G14", "G15"] {
        let doc = read_and_normalize(&fixture(case)).unwrap();
        assert!(
            resonance_scope_diagnostics(&doc).is_empty(),
            "{case}: {:?}",
            resonance_scope_diagnostics(&doc)
        );
    }
    for (case, expected) in [
        ("G07", "graph.result_scope"),
        ("G08", "graph.owner_scope"),
        ("G09", "graph.duplicate_admission"),
        ("G11", "graph.use_after_end"),
    ] {
        let doc = read_and_normalize(&fixture(case)).unwrap();
        let errors = resonance_scope_diagnostics(&doc);
        assert!(
            errors.iter().any(|e| e.code == expected),
            "{case}: {errors:?}"
        );
    }
}

#[test]
fn portable_ir_reloads_through_the_production_skill_loader() {
    use crate::game::ability::resonance::*;
    let full = compile(&fixture("G01"), "compiled_use".into()).unwrap();
    assert_eq!(
        full.operation(),
        &GraphOperation::ResonanceUse {
            admission: ResonanceAdmission {
                capacity: 100,
                requirement: ResonanceRequirement::Full,
                cost: ResonanceCost::All
            }
        }
    );
    for id in ["G01", "G02", "G14", "G15"] {
        let artifact = compile(&fixture(id), "compiled_use".into()).unwrap();
        let loaded = PortableGraph::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(loaded.operation(), artifact.operation());
        let database = loaded.load_native().unwrap();
        let native = database.get_by_id("compiled_use").unwrap();
        assert_eq!(
            native.resonance_admission,
            Some(artifact.operation().admission())
        );
        assert!(native.steps.iter().all(|s| s.effects.is_empty()));
        assert!(database.compiled_flow("compiled_use").is_ok());
        if id != "G02" {
            assert_eq!(full.operation(), loaded.operation());
        }
        if id == "G14" {
            assert!(loaded
                .source_map()
                .values()
                .flatten()
                .any(|l| l.node.as_deref().is_some_and(|n| n.contains("route"))));
        }
    }
}

#[test]
fn compiler_refuses_unsupported_unfinished_and_out_of_range_sources_without_artifacts() {
    for id in [
        "G03", "G04", "G05", "G06", "G07", "G08", "G09", "G10", "G11", "G12", "G13", "G16",
    ] {
        assert!(
            compile(&fixture(id), "compiled_use".into()).is_err(),
            "{id}"
        );
    }
    let mut doc: Value = serde_json::from_str(&fixture("G01")).unwrap();
    doc["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["id"] == "capacity")
        .unwrap()["values"]["value"] = json!(u64::from(u32::MAX) + 1);
    assert!(compile(&doc.to_string(), "compiled_use".into())
        .unwrap_err()
        .iter()
        .any(|e| e.code == "graph.numeric_target_range"));
}

#[test]
fn artifact_loader_revalidates_version_target_policy_and_strict_schema() {
    let original = compile(&fixture("G01"), "compiled_use".into()).unwrap();
    let mut wrong = original.clone();
    wrong.version += 1;
    assert!(wrong.load_native().is_err());
    assert!(PortableGraph::from_ron(&ron::ser::to_string(&wrong).unwrap()).is_err());
    let mut wrong = original.clone();
    wrong.target = "unknown".into();
    assert!(wrong.load_native().is_err());
    let mut wrong = original.clone();
    let GraphOperation::ResonanceUse { admission } = &mut wrong.operation else {
        panic!("fixture operation changed");
    };
    admission.capacity = 0;
    assert!(wrong.load_native().is_err());
    let mut value = serde_json::to_value(&original).unwrap();
    value["pretendValidated"] = json!(true);
    assert!(serde_json::from_value::<PortableGraph>(value).is_err());
}

#[test]
fn integer_admission_applies_ceiling_before_minimum_without_overflow() {
    use crate::game::ability::resonance::*;
    let mut policy = ResonanceAdmission {
        capacity: 100,
        requirement: ResonanceRequirement::Payable,
        cost: ResonanceCost::CurrentPercent {
            percent: 25,
            minimum: 0,
        },
    };
    assert_eq!(policy.payment(30, 100), Some(8));
    assert_eq!(policy.payment(22, 100), Some(6));
    assert_eq!(policy.payment(1, 100), Some(1));
    policy.cost = ResonanceCost::CurrentPercent {
        percent: 0,
        minimum: 2,
    };
    assert_eq!(policy.payment(1, 100), None);
    assert_eq!(policy.payment(3, 100), Some(2));
    policy.capacity = u32::MAX;
    policy.cost = ResonanceCost::MaximumPercent {
        percent: 100,
        minimum: 0,
    };
    assert_eq!(policy.payment(u32::MAX, u32::MAX), Some(u32::MAX));
    assert_eq!(policy.payment(100, 101), None);
}

pub(crate) fn prepared_basic_fixture() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../content_assembler/content/scarecrow-searching-for-wisdom.node-graph.json"),
    )
    .unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-next-basic-plan" => node["values"]["multiplier"] = json!(2.5),
            "skill-seconds" => node["values"]["value"] = json!(2),
            "skill-attribute-effect" => node["values"]["amount"] = json!(25),
            _ => {}
        }
    }
    doc
}
#[test]
fn prepared_basic_graph_preserves_ratio_and_optional_reward() {
    let graph = compile(&prepared_basic_fixture().to_string(), "prepared".into()).unwrap();
    let ron = graph.to_ron().unwrap();
    let restored = PortableGraph::from_ron(&ron).unwrap();
    assert_eq!(graph.operation(), restored.operation());
    let GraphOperation::ResonancePreparedBasic { profile, .. } = graph.operation() else {
        panic!("wrong operation");
    };
    assert_eq!((profile.numerator, profile.denominator), (5, 2));
    assert_eq!(profile.kill_reward.as_ref().unwrap().duration_ms, 2000);
    assert_eq!(
        profile.kill_reward.as_ref().unwrap().profile.attack_percent,
        25
    );
    let native = restored.load_native().unwrap();
    assert!(matches!(
        native.get_by_id("prepared").unwrap().use_lifetime,
        crate::game::ability::SkillUseLifetime::Action
    ));
    assert!(PortableGraph::from_ron(&ron.replace("denominator: 2", "denominator: 0")).is_err());
}

pub(crate) fn parry_fixture() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../content_assembler/content/dream-of-black-swan.node-graph.json"),
    )
    .unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-passive-guard" => node["values"]["percent"] = json!(30),
            "skill-direct-reflection" => {
                node["values"]["chance"] = json!(100);
                node["values"]["percent"] = json!(50);
            }
            "skill-guard-charge" => { node["values"]["capacity"]=json!(1000);node["values"]["factor"]=json!(2);node["values"]["splash"]=json!(0.5); }
            "skill-range" => {node["values"]["pattern"]=json!("XXX\nX@X\nXXX");node["values"]["includeAnchor"]=json!(true);}
            "skill-weapon-power" => node["values"]["coefficient"] = json!(2.5),
            "skill-seconds" if node["values"]["value"] == "" => {
                node["values"]["value"] = json!(0.03)
            }
            _ => {}
        }
    }
    doc
}

pub(crate) fn support_fixture(channel: bool) -> Value {
    let name = if channel {
        "nameless-fetus"
    } else {
        "porccubus"
    };
    let text = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        format!("../content_assembler/content/{name}.node-graph.json"),
    ))
    .unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    let selection_range = {
        let modifier = doc["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "skill-modifier-single-support")
            .unwrap()["id"]
            .as_str()
            .unwrap();
        doc["edges"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["target"]["node"] == modifier && e["target"]["port"] == "range")
            .unwrap()["source"]["node"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-seconds" => node["values"]["value"] = json!(0.2),
            "skill-personal-shield-effect" => node["values"]["percent"] = json!(20),
            "skill-modifier-single-support" if !channel => node["values"]["factor"] = json!(2.5),
            "skill-range" => {
                node["values"]["includeAnchor"] = json!(true);
                node["values"]["pattern"] = json!(if node["title"].as_str().is_some_and(|s| s.starts_with("울림통 · 반경")) {
                    "XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"
                } else if node["id"] == selection_range {
                    "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"
                } else {
                    "XXX\nX@X\nXXX"
                });
            }
            _ => (),
        }
    }
    doc
}
#[test]
fn support_graph_compiles_both_actual_plans_and_conditional_ranges() {
    for channel in [false, true] {
        let graph = compile(&support_fixture(channel).to_string(), "support".into()).unwrap();
        let ron = graph.to_ron().unwrap();
        let loaded = PortableGraph::from_ron(&ron).unwrap();
        assert_eq!(loaded.operation(), graph.operation());
        let GraphOperation::ResonanceSupport { program, .. } = loaded.operation() else {
            panic!("support program")
        };
        assert_eq!(program.range.rows.len(), 3);
        assert_eq!(
            program
                .conversion
                .as_ref()
                .unwrap()
                .selection_range
                .rows
                .len(),
            5
        );
        let native = loaded.load_native().unwrap();
        assert_eq!(native.skills.len(), 1);
        assert!(
            PortableGraph::from_ron(&ron.replace("duration_ms: 200", "duration_ms: 0")).is_err()
        );
    }
}
#[test]
fn parry_graph_binds_guard_and_counter_and_preserves_ron() {
    let graph = compile(&parry_fixture().to_string(), "parry".into()).unwrap();
    let ron = graph.to_ron().unwrap();
    let loaded = PortableGraph::from_ron(&ron).unwrap();
    assert_eq!(graph.operation(), loaded.operation());
    let GraphOperation::ResonanceParry { program, .. } = loaded.operation() else {
        panic!("wrong operation")
    };
    assert_eq!(
        (
            program.window_ms,
            program.invulnerable_ms,
            program.miss_recovery_ms
        ),
        (30, 500, 30)
    );
    assert_eq!(
        (
            program.coefficient_numerator,
            program.coefficient_denominator
        ),
        (5, 2)
    );
    let native = loaded.load_native().unwrap();
    assert_eq!(
        native.get_by_id("parry").unwrap().fragment_guard(),
        Some(program.guard)
    );
    assert!(native.get_by_id("parry:counter").is_some());
    assert!(PortableGraph::from_ron(&ron.replace("window_ms: 30", "window_ms: 0")).is_err());
}

pub(crate) fn ground_fixture(heal: bool) -> Value {
    let name = if heal { "ground-heal" } else { "ground-damage" };
    let text = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../content_assembler/content/{name}.node-graph.json"))).unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    let periodic = doc["nodes"].as_array().unwrap().iter().find(|n| n["kind"] == "skill-ground-periodic").unwrap()["id"].as_str().unwrap().to_owned();
    let interval = doc["edges"].as_array().unwrap().iter().find(|e| e["target"]["node"] == periodic && e["target"]["port"] == "interval").unwrap()["source"]["node"].as_str().unwrap().to_owned();
    let placement = doc["nodes"].as_array().unwrap().iter().find(|n| n["kind"] == "skill-range-placement").unwrap()["id"].as_str().unwrap().to_owned();
    let footprint = doc["edges"].as_array().unwrap().iter().find(|e| e["target"]["node"] == placement && e["target"]["port"] == "range").unwrap()["source"]["node"].as_str().unwrap().to_owned();
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-seconds" => node["values"]["value"] = json!(if node["id"] == interval {0.1} else {0.3}),
            "skill-ground-periodic" => node["values"]["first"] = json!("immediate"),
            "skill-ground-damage" => node["values"]["damageType"] = json!("true"),
            "skill-damage-amount" | "skill-healing-amount" => node["values"]["value"] = json!(50),
            "skill-range" => {
                node["values"]["includeAnchor"] = json!(true);
                node["values"]["pattern"] = json!(if node["id"] == footprint { "@X" } else { "XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX" });
            }
            _ => (),
        }
    }
    doc
}
#[test]
fn ground_graph_compiles_both_actual_periodic_examples_and_rejects_wrong_wiring() {
    for heal in [false, true] {
        let doc = ground_fixture(heal);
        let graph = compile(&doc.to_string(), "ground".into()).unwrap();
        let native = PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap().load_native().unwrap();
        assert_eq!(native.skills.len(), 1);
        let GraphOperation::ResonanceGround { program, .. } = graph.operation() else { panic!() };
        assert_eq!(program.duration_ms, 300);
        assert_eq!(program.interval_ms, 100);
        assert_eq!(program.footprint.rows, vec!["@X"]);
        let mut invalid = doc.clone();
        let ended = invalid["nodes"].as_array().unwrap().iter().find(|n| n["kind"] == "skill-ground-ended").unwrap()["id"].clone();
        let targets = invalid["nodes"].as_array().unwrap().iter().find(|n| n["kind"] == "skill-ground-targets").unwrap()["id"].clone();
        for edge in invalid["edges"].as_array_mut().unwrap() {
            if edge["target"]["node"] == targets && edge["target"]["port"] == "in" { edge["source"]["node"] = ended.clone(); }
        }
        assert!(compile(&invalid.to_string(), "ground".into()).is_err());
        let mut invalid = doc.clone();
        for node in invalid["nodes"].as_array_mut().unwrap() {
            if node["kind"] == "skill-ground-periodic" { node["values"]["first"] = json!("unresolved"); }
        }
        assert!(compile(&invalid.to_string(), "ground".into()).is_err());
    }
}

pub(crate) fn ground_status_fixture() -> Value {
    let text = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/ground-status.node-graph.json")).unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    let dwell = doc["nodes"].as_array().unwrap().iter().find(|n| n["kind"] == "skill-ground-dwell").unwrap()["id"].clone();
    let duration = doc["edges"].as_array().unwrap().iter().find(|e| e["target"]["node"] == dwell && e["target"]["port"] == "duration").unwrap()["source"]["node"].clone();
    for node in doc["nodes"].as_array_mut().unwrap() {
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount" => node["values"]["value"] = json!(100),
            "skill-seconds" => node["values"]["value"] = json!(if node["id"] == duration { 0.1 } else { 0.3 }),
            "skill-range" => { node["values"]["includeAnchor"] = json!(true); node["values"]["pattern"] = json!("XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"); },
            "skill-ground-status" => {
                node["values"]["lifetime"] = json!("membership");
                if node["values"]["effect"] == "slow" { node["values"]["percent"] = json!(20); node["values"]["overlap"] = json!("sum"); }
            },
            _ => (),
        }
    }
    doc
}
#[test]
fn ground_status_graph_roundtrip_keeps_entry_dwell_and_hidden_stun_fields() {
    let doc = ground_status_fixture();
    let graph = compile(&doc.to_string(), "ground".into()).unwrap();
    let ron = graph.to_ron().unwrap();
    let loaded = PortableGraph::from_ron(&ron).unwrap();
    assert_eq!(loaded.operation(), graph.operation());
    let GraphOperation::ResonanceGroundStatus { program, .. } = loaded.operation() else { panic!() };
    assert_eq!(program.status.statuses.len(), 2);
    assert!(program.status.rules.iter().any(|r| r.trigger == crate::game::ability::ground::GroundTrigger::Dwell { duration_ms: 100 }));
    assert!(PortableGraph::from_ron(&ron.replace("duration_ms: 100", "duration_ms: 0")).is_err());
    let mut wrong = doc.clone();
    let enter = wrong["nodes"].as_array().unwrap().iter().find(|n| n["kind"] == "skill-ground-enter").unwrap()["id"].clone();
    let dwell = wrong["nodes"].as_array().unwrap().iter().find(|n| n["kind"] == "skill-ground-dwell").unwrap()["id"].clone();
    for edge in wrong["edges"].as_array_mut().unwrap() { if edge["source"]["node"] == dwell && edge["source"]["port"] == "member" { edge["source"]["node"] = enter.clone(); } }
    assert!(compile(&wrong.to_string(), "ground".into()).is_err());
}

#[test]
fn ground_status_numeric_limits_match_native_signed_percentages() {
    let mut doc = ground_status_fixture();
    for percent in [10_001u32, i32::MAX as u32, i32::MAX as u32 + 1] {
        for node in doc["nodes"].as_array_mut().unwrap() {
            if node["kind"] == "skill-ground-status" && node["values"]["effect"] == "slow" { node["values"]["percent"] = json!(percent); }
        }
        assert_eq!(compile(&doc.to_string(), "ground".into()).is_ok(), percent <= i32::MAX as u32);
    }
}

pub(crate) fn strike_fixture() -> Value {
    let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/warm-hearted-woodsman.node-graph.json")).unwrap();
    let mut doc:Value=serde_json::from_str(&text).unwrap();
    let statuses:Vec<String>=doc["nodes"].as_array().unwrap().iter().filter(|n|n["kind"]=="skill-hit-status").map(|n|n["id"].as_str().unwrap().to_string()).collect();
    let duration=doc["edges"].as_array().unwrap().iter().find(|e|statuses.contains(&e["target"]["node"].as_str().unwrap().to_string())&&e["target"]["port"]=="duration").unwrap()["source"]["node"].as_str().unwrap().to_string();
    for n in doc["nodes"].as_array_mut().unwrap() {
        match n["kind"].as_str().unwrap() {
            "skill-resonance-amount"=>n["values"]["value"]=json!(100),
            "skill-seconds"=>if n["values"]["value"]=="" {n["values"]["value"]=if n["id"]==duration {json!(0.3)}else{json!(0.1)};},
            "skill-range"=>{n["values"]["pattern"]=json!("XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX");n["values"]["includeAnchor"]=json!(true);}
            "skill-weapon-power"=>n["values"]["coefficient"]=json!(1),
            "skill-missing-hp-power"=>{n["values"]["percent"]=json!(100);n["values"]["cap"]=json!(100);}
            "skill-hit-pull"=>n["values"]["tiles"]=json!(1),
            "skill-modifier-control-vulnerability"=>n["values"]["percent"]=json!(20),
            _=>(),
        }
    }
    doc
}
#[test]
fn strike_graph_ron_preserves_sequence_statuses_and_conditional_sites() {
    let graph=compile(&strike_fixture().to_string(),"strike".into()).unwrap();
    let restored=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();
    assert_eq!(graph.operation(),restored.operation());
    let GraphOperation::ResonanceStrike{program,..}=restored.operation() else {panic!("wrong operation")};
    assert_eq!(program.parts.len(),5);assert_eq!(program.policy.modifiers.len(),7);
    let native=restored.load_native().unwrap();
    assert_eq!(native.skills[0].steps.len(),3);
    assert!(matches!(&native.skills[0].steps[2].effects[0].op,crate::game::ability::SkillEffectDef::DamageExpr{amount,..} if amount.moment==crate::game::value_expr::EvalMoment::Live));
}
#[test]
fn strike_graph_rejects_unbound_modifier_and_outside_limited_pattern() {
    let mut doc=strike_fixture();
    let id=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-limited-range").unwrap()["id"].clone();
    let range=doc["edges"].as_array().unwrap().iter().find(|e|e["target"]["node"]==id&&e["target"]["port"]=="base").unwrap()["source"]["node"].clone();
    doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["id"]==range).unwrap()["values"]["pattern"]=json!("@..X");
    assert!(compile(&doc.to_string(),"strike".into()).is_err());
    let mut doc=strike_fixture();
    let entry=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-action-entry").unwrap()["id"].clone();
    let status=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-hit-status").unwrap()["id"].clone();
    let modifier=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-modifier-control-vulnerability").unwrap()["id"].clone();
    let edge=doc["edges"].as_array_mut().unwrap().iter_mut().find(|e|e["target"]["node"]==modifier&&e["source"]["node"]==status).unwrap();
    edge["source"]["node"]=entry;edge["source"]["port"]=json!("program");
    assert!(compile(&doc.to_string(),"strike".into()).is_err());
}

pub(crate) fn taunt_fixture() -> Value {
    let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/mountain-of-smiling-bodies.node-graph.json")).unwrap();
    let mut doc:Value=serde_json::from_str(&text).unwrap();
    let growth=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-weapon-kill-growth").unwrap()["id"].clone();
    let quantities:Vec<_>=doc["edges"].as_array().unwrap().iter().filter(|e|e["target"]["node"]==growth).map(|e|(e["source"]["node"].clone(),e["target"]["port"].clone())).collect();
    let replacement=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-modifier-range").unwrap()["id"].clone();
    let replacement=doc["edges"].as_array().unwrap().iter().find(|e|e["target"]["node"]==replacement&&e["target"]["port"]=="replacement").unwrap()["source"]["node"].clone();
    for n in doc["nodes"].as_array_mut().unwrap() {
        match n["kind"].as_str().unwrap() {
            "skill-resonance-amount"=>n["values"]["value"]=json!(100),
            "skill-seconds"=>n["values"]["value"]=json!(1.5),
            "skill-range"=>{n["values"]["pattern"]=if n["id"]==replacement {json!("XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX")}else{json!("XXX\nX@X\nXXX")};n["values"]["includeAnchor"]=json!(true);},
            "skill-target-speed"=>n["values"]["percent"]=json!(100),
            "skill-modifier-taunt-damage"=>n["values"]["percent"]=if n["values"]["mode"]=="taken"{json!(20)}else{json!(30)},
            "skill-weapon-kill-growth"=>{n["values"]["damage"]=json!(10);n["values"]["secondaryPercent"]=json!(40);},
            "skill-integer-amount"=>{let p=&quantities.iter().find(|(id,_)|*id==n["id"]).unwrap().1;n["values"]["value"]=match p.as_str().unwrap(){"normal"=>json!(2),"elite"=>json!(5),_=>json!(9)};},
            _=>(),
        }
    }
    doc
}
#[test]
fn taunt_graph_ron_keeps_equipment_companion_and_complete_effect_bundle() {
    let graph=compile(&taunt_fixture().to_string(),"taunt".into()).unwrap();
    let restored=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();
    assert_eq!(graph.operation(),restored.operation());
    assert!(restored.load_native().unwrap_err().contains("equipment companions"));
    let content=restored.load_content().unwrap();
    assert_eq!(content.equipment.len(),1);
    assert_eq!(content.equipment[0].weapon,"laughter");
    assert_eq!(content.equipment[0].growth.normal,2);
    assert_eq!(content.equipment[0].growth.multiplier.as_ref().unwrap().factor,2);
    assert_eq!(content.skills.skills[0].action_policy.as_ref().unwrap().modifiers.len(),4);
    assert!(matches!(content.skills.skills[0].steps[0].effects[0].op,crate::game::ability::SkillEffectDef::Sustained { definition:crate::game::ability::sustained::SustainedEffectDef::Control{duration_ms:1500,kind:crate::game::ability::sustained::ControlKind::ApproachTaunt {..}} }));
}
#[test]
fn taunt_graph_rejects_partial_state_wrong_lifetime_and_partial_growth_multiplier() {
    for (kind,field,value) in [("skill-target-state","sourceExit",json!("independent")),("skill-target-speed","percent",json!(0)),("skill-weapon-kill-growth","damage",json!(0)),("skill-modifier-number","factor",json!(3))] {
        let mut doc=taunt_fixture();
        doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]==kind).unwrap()["values"][field]=value;
        assert!(compile(&doc.to_string(),"taunt".into()).is_err(),"{kind} {field}");
    }
    let mut doc=taunt_fixture();
    let speed=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-target-speed").unwrap()["id"].clone();
    doc["edges"].as_array_mut().unwrap().retain(|e|e["source"]["node"]!=speed);
    assert!(compile(&doc.to_string(),"taunt".into()).is_err());
}

pub(crate) fn channel_ground_fixture() -> Value {
    let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/burrowing-heaven.node-graph.json")).unwrap();
    let mut doc:Value=serde_json::from_str(&text).unwrap();
    let id=|kind:&str|doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]==kind).unwrap()["id"].clone();
    let channel=id("skill-channel-begin");let ramp=id("skill-ground-ramping-slow");let modifier=id("skill-modifier-range");
    let source=|node:&Value,port:&str|doc["edges"].as_array().unwrap().iter().find(|e|&e["target"]["node"]==node&&e["target"]["port"]==port).unwrap()["source"]["node"].clone();
    let channel_duration=source(&channel,"duration");let ramp_duration=source(&ramp,"duration");let replacement=source(&modifier,"replacement");
    for n in doc["nodes"].as_array_mut().unwrap() {
        match n["kind"].as_str().unwrap() {
            "skill-resonance-amount"=>n["values"]["value"]=json!(100),
            "skill-seconds" if n["id"]==channel_duration=>n["values"]["value"]=json!(0.6),
            "skill-seconds" if n["id"]==ramp_duration=>n["values"]["value"]=json!(0.2),
            "skill-range"=>{n["values"]["pattern"]=if n["id"]==replacement {json!("XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX")}else{json!("XXX\nX@X\nXXX")};n["values"]["includeAnchor"]=json!(true);},
            "skill-ground-ramping-slow"=>n["values"]["maxSlow"]=json!(60),
            "skill-modifier-ramp"=>n["values"]["factor"]=json!(2),
            "skill-modifier-control-vulnerability"=>n["values"]["percent"]=json!(30),
            _=>(),
        }
    }
    doc
}
#[test]
fn channel_ground_graph_ron_keeps_ramp_clock_channel_ownership_and_five_modifiers() {
    let graph=compile(&channel_ground_fixture().to_string(),"channel".into()).unwrap();
    let restored=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();assert_eq!(graph.operation(),restored.operation());
    let GraphOperation::ResonanceChannelGround{program,..}=restored.operation() else {panic!("wrong operation")};
    assert_eq!(program.policy.modifiers.len(),5);assert_eq!(program.status.statuses.len(),3);assert_eq!(program.duration_ms,600);
    let native=restored.load_native().unwrap();
    assert!(matches!(&native.skills[0].steps[0].delivery,crate::game::ability::DeliveryDef::TileArea{area,..} if matches!(area.mode,crate::game::ability::SkillTileAreaMode::ChannelGround{pause_basic:true})));
}
#[test]
fn channel_ground_graph_rejects_wrong_anchor_unhandled_cancel_and_foreign_progress() {
    for (kind,field,value) in [("skill-range-placement","positionTime",json!("release")),("skill-ground-ramping-slow","maxSlow",json!(101)),("skill-modifier-ramp","factor",json!(0))] {
        let mut doc=channel_ground_fixture();doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]==kind).unwrap()["values"][field]=value;
        assert!(compile(&doc.to_string(),"channel".into()).is_err(),"{kind}");
    }
    let mut doc=channel_ground_fixture();let id=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-channel-cancel").unwrap()["id"].clone();
    doc["edges"].as_array_mut().unwrap().retain(|e|e["target"]["node"]!=id||e["target"]["port"]!="in");
    assert!(compile(&doc.to_string(),"channel".into()).is_err());
}

pub(crate) fn presence_fixture() -> Value {
    let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/melting-love.node-graph.json")).unwrap();
    let mut doc:Value=serde_json::from_str(&text).unwrap();
    for n in doc["nodes"].as_array_mut().unwrap() {
        match n["kind"].as_str().unwrap() {
            "skill-resonance-amount"=>n["values"]["value"]=json!(100),
            "skill-seconds"=>n["values"]["value"]=json!(1),
            "skill-population-power"=>{n["values"]["growth"]=json!(10);n["values"]["cap"]=json!(100);n["values"]["decay"]=json!(2);},
            "skill-mark-spread"=>{n["values"]["radius"]=json!(2);n["values"]["limit"]=json!(1);},
            "skill-mark-weapon-hit"=>n["values"]["chance"]=json!(100),
            "skill-modifier-duration"=>n["values"]["percent"]=json!(-50),
            "skill-modifier-explosion-power"=>n["values"]["percent"]=json!(20),
            "skill-modifier-population" if n["values"]["mode"]=="growth"=>n["values"]["percent"]=json!(50),
            _=>(),
        }
    }doc
}
#[test]
fn presence_graph_roundtrip_preserves_population_and_six_explicit_modifiers() {
    let graph=compile(&presence_fixture().to_string(),"presence".into()).unwrap();
    let restored=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();assert_eq!(graph.operation(),restored.operation());
    let native=restored.load_native().unwrap();
    let crate::game::ability::SkillEffectDef::PresenceBurst{program}=&native.skills[0].steps[0].effects[0].op else {panic!()};
    assert_eq!(program.marks.power.growth_per_second.numerator,10);assert_eq!(program.modifiers.len(),6);
    assert_eq!(program.marks.periodic_interval_ms,1000);
}
#[test]
fn presence_graph_rejects_mixed_ownership_and_unresolved_game_values() {
    for (kind,field,value) in [("skill-target-marks","lifetime",json!("run")),("skill-mark-weapon-hit","binding",json!("weapon")),("skill-population-power","growth",json!("")),("skill-mark-spread","limit",json!(0)),("skill-modifier-duration","percent",json!(-100))] {
        let mut doc=presence_fixture();doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]==kind).unwrap()["values"][field]=value;
        assert!(compile(&doc.to_string(),"invalid".into()).is_err(),"{kind}.{field}");
    }
}

#[test]
fn presence_native_loader_rejects_delayed_or_repeated_consumption_and_allows_zero_growth() {
    let mut doc=presence_fixture();doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-population-power").unwrap()["values"]["growth"]=json!(0);
    let graph=compile(&doc.to_string(),"presence".into()).unwrap();let native=graph.load_native().unwrap();
    for delayed in [true,false] {
        let mut skill=native.skills[0].clone();
        if delayed {skill.steps[0].delay_ms=1;}else{let mut duplicate=skill.steps[0].clone();duplicate.id="duplicate".into();skill.steps.push(duplicate);}
        assert!(crate::game::data::skill_data::SkillDatabase::from_ron("invalid-presence",&format!("(skills:[{}])",ron::ser::to_string(&skill).unwrap())).is_err());
    }
}


/// Explicit test values; authored files retain their unresolved balance inputs.
pub(crate) fn tile_burst_fixture(form: &str) -> Value {
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../content_assembler/content/funeral-of-dying-butterflies{form}.node-graph.json"));
    let mut doc:Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        let title=node["title"].as_str().unwrap_or("").to_owned();
        match node["kind"].as_str().unwrap() {
            "skill-resonance-amount"=>node["values"]["value"]=json!(100),
            "skill-damage-amount"=>node["values"]["value"]=json!(10),
            "skill-seconds"=>node["values"]["value"]=json!(if title.contains("연출") {0.5}else{0.1}),
            "skill-mark-weapon-hit"=>{node["values"]["weapon"]=json!("r11_weapon");if node["values"]["binding"]=="weapon-base" {node["values"]["amount"]=json!(2);}},
            "skill-weapon-evolution"=>{node["values"]["weapon"]=json!("r11_weapon");node["values"]["fragment"]=json!("r09_fragment");},
            "skill-stack-grade"=>{node["values"]["divisor"]=json!(3);node["values"]["rankBonus"]=json!(50);},
            "skill-modifier-explosion-power"=>node["values"]["percent"]=json!(50),
            "skill-burst-conversion"=>{node["values"]["threshold"]=json!(6);node["values"]["perStack"]=json!(2);},
            "skill-range" if title.contains("구체")=>{node["values"]["pattern"]=json!("XXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX");node["values"]["includeAnchor"]=json!(true);},
            "skill-explosion-rule"=>node["values"]["damageType"]=json!("true"),
            _=>{}
        }
    }
    doc
}
#[test]
fn tile_burst_graph_roundtrip_preserves_weapon_companion_and_rejects_skill_only_load() {
    for form in ["","-bloom","-orb"] {
        let graph=compile(&tile_burst_fixture(form).to_string(),"r11_aura".into()).unwrap();
        let restored=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();
        assert_eq!(graph.operation(),restored.operation());
        assert!(restored.load_native().is_err());
        let content=restored.load_content().unwrap();
        assert_eq!(content.mark_weapons.len(),1);
        assert_eq!(content.mark_weapons[0].definition.base.amount,2);
        assert_eq!(content.mark_weapons[0].definition.upgraded.amount,1);
        assert_eq!(content.skills.skills.len(),1);
    }
}

#[test]
fn tile_burst_catalog_binding_keeps_one_weapon_owner_and_refuses_conflicts() {
    use crate::game::data::{equipment_data::*, skill_data::SkillDatabase};
    let graph = compile(&tile_burst_fixture("-bloom").to_string(), "r11_aura".into()).unwrap();
    let content = graph.load_content().unwrap();
    let weapon = EquipmentMetadata {
        content_family: None,
        id: "r11_weapon".into(),
        uuid: uuid::Uuid::from_u128(0x1101),
        name: "Fixture".into(),
        equipment_type: EquipmentType::Weapon,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        price: 0,
        allow_duplicate_equip: true,
        bound: false,
        cannot_unequip_reason: "equipment_bound".into(),
        triggered_effects: Default::default(),
        ability_activations: vec![],
        weapon_profile: Some(Default::default()),
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    };
    let equipment = EquipmentDatabase::new(vec![weapon]);
    let skills = SkillDatabase::new(vec![]);
    let resources = crate::game::resource::ResourceCatalog { resources: vec![] };
    let bound = content
        .bind_catalogs(&skills, &equipment, &resources)
        .unwrap();
    assert_eq!(
        bound.equipment.items[0]
            .weapon_profile
            .as_ref()
            .unwrap()
            .mark_weapon
            .as_ref(),
        Some(&content.mark_weapons[0].definition)
    );
    assert!(
        equipment.items[0]
            .weapon_profile
            .as_ref()
            .unwrap()
            .mark_weapon
            .is_none(),
        "input catalog stays unchanged"
    );
    assert!(
        content
            .bind_catalogs(&bound.skills, &bound.equipment, &bound.resources)
            .is_err(),
        "duplicate skills cannot be published"
    );
    let mut conflicting = bound.equipment.clone();
    conflicting.items[0]
        .weapon_profile
        .as_mut()
        .unwrap()
        .mark_weapon
        .as_mut()
        .unwrap()
        .base
        .amount += 1;
    assert!(content
        .bind_catalogs(&skills, &conflicting, &resources)
        .is_err());
    assert!(content
        .bind_catalogs(&skills, &EquipmentDatabase::new(vec![]), &resources)
        .is_err());
    let mut invalid = graph.clone();
    let GraphOperation::ResonanceTileBurst { program, .. } = &mut invalid.operation else {
        panic!()
    };
    let crate::game::ability::tile_burst::TileBurstSource::WeaponStacks { key, .. } =
        &mut program.effect.source
    else {
        panic!()
    };
    *key = "another-store".into();
    assert!(invalid.to_ron().is_err());
}


pub(crate) fn counter_channel_fixture(bloom: bool) -> Value {
    let name=if bloom {"queen-of-hatred-bloom"} else {"queen-of-hatred"};
    let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../content_assembler/content/{name}.node-graph.json"))).unwrap();
    let mut doc:Value=serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        let k=node["kind"].as_str().unwrap().to_owned();
        let title=node["title"].as_str().unwrap().to_owned();
        let v=&mut node["values"];
        match k.as_str() {
            "skill-resonance-amount"=>v["value"]=json!(100),
            "skill-range"=>{v["pattern"]=json!("XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX");v["includeAnchor"]=json!(true);},
            "skill-seconds"=>if !v["value"].is_number(){v["value"]=json!(if title.contains("히스테릭") {1.0}else{0.3});},
            "skill-ground-pull"=>v["tiles"]=json!(0.5),
            "skill-damage-amount"=>v["value"]=json!(10),
            "skill-healing-amount"=>v["value"]=json!(8),
            "skill-count-damage"|"skill-count-healing"=>v["percent"]=json!(25),
            "skill-modifier-duration"=>v["percent"]=json!(-20),
            "skill-modifier-healing"=>v["percent"]=json!(25),
            "skill-attribute-effect"=>{v["attribute"]=json!("attack");v["operation"]=json!("percent");v["amount"]=json!(10);},
            _=>{}
        }
    }
    doc
}
#[test]
fn counter_channel_graph_preserves_menu_counter_phases_and_ron() {
    use crate::game::ability::counter_channel::*;
    for bloom in [false,true] {
        let graph=compile(&counter_channel_fixture(bloom).to_string(),"counter".into()).unwrap();
        let loaded=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();
        assert_eq!(loaded.operation(),graph.operation());
        let GraphOperation::ResonanceCounterChannel{program}=loaded.operation() else {panic!("wrong operation")};
        assert_eq!(program.capacity,6);assert_eq!(program.interval_ms,15000);assert_eq!(program.periodic_gain,1);
        assert_eq!(program.entries.len(),3);
        assert!(program.modifiers.iter().any(|m|matches!(m.change,CounterChange::Period{decrease:true,percent:crate::game::ability::marks::MarkRatio{numerator:20,denominator:1}})));
        let beam=&program.entries[program.default_entry];
        assert!(beam.remember_aim);assert_eq!(beam.phases[0].duration_ms,2000);assert_eq!(beam.phases[0].interval_ms,500);
        assert_eq!(beam.completion_award.as_ref().unwrap().damage_and_healing_bonus,1);
        let manual=program.entries.iter().find(|e|e.spend==CounterSpend::AdmissionAll).unwrap();
        assert_eq!(manual.minimum,if bloom {0}else{3});assert_eq!(manual.maximum,5);
        assert_eq!(manual.phases.len(),if bloom {2}else{1});
        assert!(matches!(manual.phases[0].queries[0].effect,PhaseEffect::Pull{distance_units:500000}));
        assert!(matches!(manual.phases[0].queries[1].effect,PhaseEffect::Heal{..}));
        let auto=program.entries.iter().find(|e|e.automatic).unwrap();assert_eq!(auto.phases.len(),2);
        assert_eq!(auto.spend,CounterSpend::EndAll);assert!(auto.completion_award.is_none());
        if bloom {assert_eq!(manual.phases[1].origin,PhaseOrigin::SelectedTile);assert_eq!(manual.phases[1].duration_ms,3000);}
        else {assert_eq!(auto.phases[1].origin,PhaseOrigin::SelectedDirection);}
        assert_eq!(loaded.load_native().unwrap().skills.len(),3);
    }
}

#[test]
fn counter_channel_native_loader_requires_complete_matching_menu_and_selection_flow() {
    use crate::game::{ability::SkillEffectDef,data::skill_data::SkillDatabase};
    let graph=compile(&counter_channel_fixture(false).to_string(),"counter".into()).unwrap();
    let native=graph.load_native().unwrap();
    for fault in 0..4 {
        let mut skills=native.skills.clone();
        match fault {
            0=>{skills.pop();},
            1=>{let SkillEffectDef::CounterChannel{program,..}=&mut skills[1].steps[0].effects[0].op else {panic!()};std::sync::Arc::make_mut(program).interval_ms+=1;},
            2=>{let crate::game::ability::SkillCastTargetingDef::Explicit{target_range,..}=&mut skills[0].cast_targeting;*target_range=None;},
            _=>skills[0].flow=None,
        }
        #[derive(serde::Serialize)] struct NativeSkills { skills:Vec<crate::game::ability::SkillDef> }
        let ron=ron::ser::to_string(&NativeSkills {skills}).unwrap();
        assert!(SkillDatabase::from_ron("counter-menu-fixture",&ron).is_err(),"fault {fault}");
    }
}
#[test]
fn counter_channel_graph_rejects_cross_use_outcomes_and_missing_count_tiers() {
    for bloom in [false,true] {
        let mut doc=counter_channel_fixture(bloom);
        let tier=doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-count-range-tier").unwrap();tier["values"]["count"]=json!(99);
        assert!(compile(&doc.to_string(),"counter".into()).is_err());
        let mut doc=counter_channel_fixture(bloom);
        let nodes=doc["nodes"].as_array().unwrap();
        let summary=nodes.iter().find(|n|n["kind"]=="skill-cast-outcomes").unwrap()["id"].as_str().unwrap().to_owned();
        let wrong=nodes.iter().find(|n|n["kind"]=="skill-ground-damage" && n["title"].as_str().unwrap().contains("히스테릭")).unwrap()["id"].clone();
        let edge=doc["edges"].as_array_mut().unwrap().iter_mut().find(|e|e["target"]["node"]==summary && e["target"]["port"]=="damage").unwrap();edge["source"]["node"]=wrong;
        assert!(compile(&doc.to_string(),"counter".into()).is_err());
    }
}

pub(crate) fn attack_overlay_fixture() -> Value {
    let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/hello-goodbye.node-graph.json")).unwrap();
    let mut doc:Value=serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap() {
        let kind=node["kind"].as_str().unwrap().to_owned();
        let v=&mut node["values"];
        match kind.as_str() {
            "skill-resonance-amount"=>v["value"]=json!(100),
            "skill-seconds"=>v["value"]=json!(2),
            "skill-attack-profile"=>{v["length"]=json!(6.5);v["width"]=json!(0.5);v["power"]=json!(40);},
            "skill-time-power"=>{v["base"]=json!(10);v["rate"]=json!(20);v["curve"]=json!("capped-linear");v["cap"]=json!(45);},
            "skill-area"=>{v["pattern"]=json!("XXXXXXXXX\nXXXXXXXXX\nXXXXXXXXX\nXXXXXXXXX\nXXXX@XXXX\nXXXXXXXXX\nXXXXXXXXX\nXXXXXXXXX\nXXXXXXXXX");v["includeAnchor"]=json!(true);},
            "skill-modifier-piercing"=>{v["step"]=json!(25);v["cap"]=json!(50);v["curve"]=json!("capped-linear");},
            _=>{}
        }
    }
    doc
}
#[test]
fn attack_overlay_graph_preserves_direct_weapon_profile_reinput_and_conditional_relic() {
    let graph=compile(&attack_overlay_fixture().to_string(),"overlay".into()).unwrap();
    let loaded=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();
    assert_eq!(loaded.operation(),graph.operation());
    let GraphOperation::ResonanceAttackOverlay{program,admission}=loaded.operation() else {panic!("wrong operation")};
    assert_eq!(admission.capacity,100);
    assert_eq!(program.line.length_units,6_500_000);
    assert_eq!(program.line.power,40);
    assert_eq!(program.line.piercing.as_ref().unwrap().increment_percent,25);
    assert_eq!(program.line.piercing.as_ref().unwrap().relic_definition,"piercing-damage-increase");
    assert_eq!(program.duration_ms,2000);
    assert_eq!(program.cap,Some(45));
    let native=loaded.load_native().unwrap();
    assert!(native.skills.iter().any(|s|s.id.as_str()=="overlay"));
    assert!(native.skills.iter().any(|s|s.id.as_str()=="overlay::finish"));
}
#[test]
fn attack_overlay_graph_rejects_detached_or_expired_followup_and_inexact_dimensions() {
    for (kind,field,value) in [
        ("skill-claim","expiryOrder",json!("input-first")),
        ("skill-permission","permission",json!("active-use")),
        ("skill-capture-time","moment",json!("release")),
        ("skill-owner-exit","policy",json!("unresolved")),
        ("skill-attack-profile","length",json!(0.0000001)),
        ("skill-attack-profile","weaponEffects",json!(false)),
        ("skill-modifier-piercing","step",json!(0)),
    ] {
        let mut doc=attack_overlay_fixture();
        let n=doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]==kind).unwrap();
        n["values"][field]=value;
        assert!(compile(&doc.to_string(),"overlay".into()).is_err(),"{kind}:{field}");
    }
    let mut doc=attack_overlay_fixture();
    let lookup=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-lookup").unwrap()["id"].clone();
    let capture=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-capture-time").unwrap()["id"].clone();
    for edge in doc["edges"].as_array_mut().unwrap() {
        if edge["target"]["node"]==capture && edge["target"]["port"]=="activation" {edge["source"]["node"]=lookup.clone();}
    }
    assert!(compile(&doc.to_string(),"overlay".into()).is_err());
}

#[test]
fn attack_overlay_linear_power_does_not_require_an_inactive_cap_and_old_damage_draft_stays_unresolved() {
    let mut doc=attack_overlay_fixture();
    let power=doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-time-power").unwrap();
    power["values"]["curve"]=json!("linear");power["values"]["cap"]=json!("");
    let graph=compile(&doc.to_string(),"overlay".into()).unwrap();
    assert!(matches!(graph.operation(),GraphOperation::ResonanceAttackOverlay{program,..} if program.cap.is_none()));
    let damage=doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-area-damage").unwrap();
    let id=damage["id"].as_str().unwrap().to_owned();damage["values"]=json!({});
    let normalized=read_and_normalize(&doc.to_string()).unwrap();
    assert!(normalized.nodes[&id].values.is_empty());
    let errors=compile(&doc.to_string(),"overlay".into()).unwrap_err();
    assert!(errors.iter().any(|e|e.code=="graph.policy_unresolved"&&e.primary.node.as_deref()==Some(&id)&&e.primary.field.as_deref()==Some("damageType")));
}

pub(crate) fn relation_action_fixture(bloom:bool)->Value {
    let name=if bloom{"knight-of-despair-bloom"}else{"knight-of-despair"};
    let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../content_assembler/content/{name}.node-graph.json"))).unwrap();
    let mut doc:Value=serde_json::from_str(&text).unwrap();
    for node in doc["nodes"].as_array_mut().unwrap(){
        let kind=node["kind"].as_str().unwrap().to_owned();let title=node["title"].as_str().unwrap().to_owned();let v=&mut node["values"];
        match kind.as_str(){
            "skill-resonance-amount"=>v["value"]=json!(100),
            "skill-range"=>if v["pattern"].as_str().is_none_or(|p|p.trim().is_empty()){v["pattern"]=json!("XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX");v["includeAnchor"]=json!(true);},
            "skill-seconds"=>if !v["value"].is_number(){v["value"]=json!(if title.contains("검 생성"){1.0}else if title.contains("총시간"){0.6}else if title.contains("기본 시간"){2.0}else if title.contains("추가 시간"){0.5}else{0.1});},
            "skill-damage-amount"=>v["value"]=json!(100),
            "skill-count-positive"=>v["value"]=json!(3),
            "skill-relation-reduction"=>v["percent"]=json!(25),
            "skill-modifier-duration"=>v["percent"]=json!(-20),
            "skill-modifier-relation-shield"=>v["percent"]=json!(50),
            "skill-modifier-control-vulnerability"=>v["percent"]=json!(10),
            _=>(),
        }
    }doc
}
#[test]
fn relation_action_graph_preserves_all_plans_and_ron_native_contract(){
    use crate::game::ability::relation_action::*;
    for bloom in [false,true]{
        let graph=compile(&relation_action_fixture(bloom).to_string(),"relation".into()).unwrap();
        let loaded=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();assert_eq!(graph.operation(),loaded.operation());
        let GraphOperation::ResonanceRelationAction{program}=loaded.operation()else{panic!()};
        assert_eq!(program.stock_capacity,Some(4));assert_eq!(program.period_ms,1000);assert_eq!(program.periodic_gain,1);
        assert_eq!(program.shield_duration_ms,15000);assert_eq!(program.normal.next_basic,!bloom);assert!(!program.active.next_basic);
        assert_eq!(program.active.self_damage.is_some(),bloom);assert_eq!(program.active.manual_stop,bloom);
        assert!(program.modifiers.iter().any(|m|matches!(m.change,RelationActionChange::UnlimitedStock)));
        assert!(program.modifiers.iter().any(|m|matches!(m.change,RelationActionChange::StatusCount{..})));
        assert_eq!(loaded.load_native().unwrap().skills.len(),1);
    }
}
#[test]
fn relation_action_graph_rejects_a_range_connected_to_a_primary_strike() {
    let mut doc = relation_action_fixture(false);
    let nodes = doc["nodes"].as_array().unwrap();
    let range = nodes.iter().find(|n| n["kind"] == "skill-range").unwrap()["id"].clone();
    let primary = nodes.iter().find(|n| {
        n["kind"] == "skill-strike-plan" && n["values"]["target"] == "primary"
    }).unwrap()["id"].clone();
    doc["edges"].as_array_mut().unwrap().push(json!({
        "id": "primary-range-regression",
        "source": { "node": range, "port": "range" },
        "target": { "node": primary, "port": "range" }
    }));
    let errors = compile(&doc.to_string(), "relation".into()).unwrap_err();
    assert!(errors.iter().any(|e| e.code == "graph.lowering_contract"
        && e.primary.node.as_deref() == primary.as_str()
        && e.primary.port.as_deref() == Some("range")), "{errors:?}");
    doc["edges"].as_array_mut().unwrap().pop();
    assert!(compile(&doc.to_string(), "relation".into()).is_ok());
}

#[test]
fn relation_action_native_load_rejects_lost_ownership_selection_and_plan_references(){
 use crate::game::{ability::{SkillEffectDef,relation_action::*},data::skill_data::SkillDatabase};
 for fault in 0..6{let native=compile(&relation_action_fixture(false).to_string(),"relation".into()).unwrap().load_native().unwrap();let mut skills=native.skills.clone();
  match fault{
   0=>skills[0].flow=None,
   1=>skills[0].use_lifetime=crate::game::ability::SkillUseLifetime::Action,
   2=>{let SkillEffectDef::RelationAction{program}=&mut skills[0].steps[0].effects[0].op else{panic!()};std::sync::Arc::make_mut(program).active.selector=None;},
   3=>{let SkillEffectDef::RelationAction{program}=&mut skills[0].steps[0].effects[0].op else{panic!()};std::sync::Arc::make_mut(program).plans[0].op=ActionPlan::Sequence{first:0,next:0};},
   4=>{let SkillEffectDef::RelationAction{program}=&mut skills[0].steps[0].effects[0].op else{panic!()};std::sync::Arc::make_mut(program).modifiers[0].change=RelationActionChange::PrimaryArea{definition:"missing-plan".into()};},
   _=>skills[0].resonance_admission=None,
  }
  #[derive(serde::Serialize)]struct Rows{skills:Vec<crate::game::ability::SkillDef>}
  assert!(SkillDatabase::from_ron("relation-native",&ron::ser::to_string(&Rows{skills}).unwrap()).is_err(),"fault {fault}");
 }
}
#[test]
fn relation_action_graph_requires_positive_base_hits_but_allows_zero_paid_repeats(){
    let mut doc=relation_action_fixture(false);
    doc["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-count-positive").unwrap()["values"]["value"]=json!(0);
    assert!(compile(&doc.to_string(),"relation".into()).is_err());
    assert!(compile(&relation_action_fixture(true).to_string(),"relation".into()).is_ok());
}

pub(crate) fn input_plan_fixture()->Value {
    let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/little-red.node-graph.json")).unwrap();
    let mut doc:Value=serde_json::from_str(&text).unwrap();
    for n in doc["nodes"].as_array_mut().unwrap(){
        let kind=n["kind"].as_str().unwrap().to_owned();let title=n["title"].as_str().unwrap().to_owned();let v=&mut n["values"];
        match kind.as_str(){
            "skill-resonance-amount"=>v["value"]=json!(100),
            "skill-stock"=>v["capacity"]=json!(8),
            "skill-range"=>{v["pattern"]=json!("XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX");v["includeAnchor"]=json!(true);},
            "skill-seconds"=>if !v["value"].is_number(){v["value"]=json!(if title.contains("출현"){5.0}else if title.contains("비행"){3.0}else{1.0});},
            "skill-damage-amount"=>v["value"]=json!(100),
            "skill-damage-bonus"=>v["bonus"]=json!(50),
            "skill-hostile-profile"=>{v["hp"]=json!(300);v["power"]=json!(20);v["speed"]=json!(1);v["physicalDefense"]=json!(0);v["magicDefense"]=json!(0);v["leak"]=json!(1);},
            "skill-projectile-profile"=>{v["speed"]=json!(5);v["range"]=json!(10);v["width"]=json!(0.5);v["hits"]=json!(3);},
            "skill-modifier-projectile-wound"=>v["cap"]=json!(5),
            "skill-modifier-projectile-push"=>v["tiles"]=json!(1),
            "skill-modifier-projectile-explosion"=>v["air"]=json!("include"),
            "skill-explosion-rule"=>v["damageType"]=json!("magic"),
            "skill-modifier-control-vulnerability"=>v["percent"]=json!(10),
            _=>(),
        }
    }doc
}
#[test]
fn input_plan_graph_compiles_stocks_branches_projectiles_and_prerequisites(){
    use crate::game::ability::input_plan::*;
    let graph=compile(&input_plan_fixture().to_string(),"volley".into()).unwrap();
    let loaded=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();assert_eq!(graph.operation(),loaded.operation());
    let GraphOperation::ResonanceInputPlan{program}=loaded.operation()else{panic!()};
    assert_eq!(program.stock_interval_ms,1000);assert_eq!(program.cohort.interval_ms,5000);
    assert_eq!(program.normal_projectile.power,100);assert_eq!(program.enhanced_projectile.power,150);
    assert_eq!(program.enhanced_projectile.after[0].duration_ms,2000);
    assert!(program.modifiers.iter().any(|m|matches!(m.change,ShotChange::WoundGrant{..})&&m.requires==vec!["little-red-puncture-wound"]));
    assert_eq!(loaded.load_native().unwrap().skills.len(),1);
}

#[test]
fn input_plan_graph_rejects_unpaired_flow_stock_capacity_and_unrepresentable_stats(){
    for case in 0..5{
        let mut d=input_plan_fixture();
        if case==0{let id=d["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-plan-commit").unwrap()["id"].clone();d["edges"].as_array_mut().unwrap().retain(|e|!(e["target"]["node"]==id && e["target"]["port"]=="in"));}
        if case==1{let n=d["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-hostile-profile").unwrap();n["values"]["speed"]=json!(0.0001);}
        if case==2{let n=d["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-hostile-profile").unwrap();n["values"]["physicalDefense"]=json!(2147483648u64);}
        if case==3{let n=d["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-stock").unwrap();n["values"]["initial"]=json!(9);}
        if case==4{let n=d["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-hostile-profile").unwrap();n["values"]["speed"]=json!(0.0015);}
        assert!(compile(&d.to_string(),"bad_plan".into()).is_err(),"{case}");
    }
}
#[test]
fn input_plan_native_load_preserves_owned_immediate_atomic_release_contract(){
    use crate::game::ability::{SkillUseLifetime,SkillEffectDef};
    let native=compile(&input_plan_fixture().to_string(),"volley".into()).unwrap().load_content().unwrap();
    for case in 0..5{
        let mut db=native.skills.clone();let skill=&mut db.skills[0];
        match case{
            0=>skill.use_lifetime=SkillUseLifetime::Action,
            1=>skill.steps[0].delay_ms=1,
            2=>skill.resonance_admission=None,
            3=>{let SkillEffectDef::InputPlan{program}=&mut skill.steps[0].effects[0].op else{panic!()};std::sync::Arc::make_mut(program).normal.capacity=Some(0);},
            _=>{let SkillEffectDef::InputPlan{program}=&mut skill.steps[0].effects[0].op else{panic!()};std::sync::Arc::make_mut(program).cohort.weakness_percent=crate::game::ability::marks::MarkRatio{numerator:3,denominator:2};},
        }
        assert!(crate::game::data::skill_data::SkillDatabase::from_ron("invalid-plan",&ron::ser::to_string(&db).unwrap()).is_err(),"{case}");
    }
}


pub(crate) fn prepared_volley_fixture()->Value {
 let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/king-of-greed.node-graph.json")).unwrap();
 let mut doc:Value=serde_json::from_str(&text).unwrap();
 for node in doc["nodes"].as_array_mut().unwrap(){let kind=node["kind"].as_str().unwrap().to_owned();let title=node["title"].as_str().unwrap().to_owned();let v=&mut node["values"];
  match kind.as_str(){
   "skill-resonance-amount"=>v["value"]=json!(100),
   "skill-range"=>{v["pattern"]=json!(if title.contains("선택"){"XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"}else{"@"});v["includeAnchor"]=json!(true);},
   "skill-seconds"=>{if !v["value"].is_number(){v["value"]=json!(if title.contains("회수"){5.0}else if title.contains("첫 포탈"){1.0}else if title.contains("마지막 포탈"){0.5}else if title.contains("발사 기한"){10.0}else if title.contains("유지 유예"){2.0}else if title.contains("비행"){5.0}else if title.contains("사출 간격"){0.25}else{1.0});}},
   "skill-damage-amount"=>v["value"]=json!(100),
   "skill-projectile-profile"=>{v["speed"]=json!(5);v["range"]=json!(20);v["width"]=json!(0.5);},
   "skill-portal-policy"=>v["maximum"]=json!(3),_=>()
  }
 }doc
}
#[test]
fn prepared_volley_graph_roundtrip_preserves_ledger_barrier_cost_and_modifiers(){
 use crate::game::ability::prepared_volley::*;
 let p=compile(&prepared_volley_fixture().to_string(),"gold".into()).unwrap();let q=PortableGraph::from_ron(&p.to_ron().unwrap()).unwrap();assert_eq!(p.operation(),q.operation());
 let GraphOperation::ResonancePreparedVolley{program}=q.operation()else{panic!()};
 assert_eq!(program.ledger.grant,10);assert_eq!(program.ledger.credit_limit,20);assert_eq!(program.policy.maximum,3);assert_eq!(program.policy.first_ms,1000);assert_eq!(program.policy.last_ms,500);assert_eq!(program.projectile.power,100);assert_eq!(program.mark_duration_ms,5000);
 assert!(program.modifiers.iter().any(|m|matches!(m.change,VolleyChange::Continuous{cost_interval_ms:1000,interval_ms:250,..})));assert_eq!(q.load_native().unwrap().skills.len(),1);
}
#[test]
fn prepared_volley_graph_rejects_zero_cadence_acceleration_reversal_and_wrong_selector(){
 for case in 0..3{let mut doc=prepared_volley_fixture();for n in doc["nodes"].as_array_mut().unwrap(){
  if case==0&&n["title"].as_str().unwrap().contains("사출 간격"){n["values"]["value"]=json!(0);}
  if case==1&&n["title"].as_str().unwrap().contains("마지막 포탈"){n["values"]["value"]=json!(2);}
  if case==2&&n["kind"]=="skill-enemy-selector"{n["values"]["order"]=json!("route");}
 }assert!(compile(&doc.to_string(),"invalid".into()).is_err(),"{case}");}
}
#[test]
fn prepared_volley_native_rejects_lost_owned_lifetime_or_delayed_step(){
 use crate::game::ability::SkillUseLifetime;
 let native=compile(&prepared_volley_fixture().to_string(),"gold".into()).unwrap().load_native().unwrap();
 for case in 0..3{let mut skills=native.clone();match case{0=>skills.skills[0].use_lifetime=SkillUseLifetime::Action,1=>skills.skills[0].steps[0].delay_ms=1,_=>skills.skills[0].resonance_admission=None};
 assert!(crate::game::data::skill_data::SkillDatabase::from_ron("gold",&ron::ser::to_string(&skills).unwrap()).is_err());}
}

pub(crate) fn paired_areas_fixture()->Value{
 let text=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../content_assembler/content/yin-yang.node-graph.json")).unwrap();let mut doc:Value=serde_json::from_str(&text).unwrap();
 for n in doc["nodes"].as_array_mut().unwrap(){let kind=n["kind"].as_str().unwrap().to_owned();let title=n["title"].as_str().unwrap().to_owned();let v=&mut n["values"];match kind.as_str(){
  "skill-resonance-amount"=>v["value"]=json!(100),
  "skill-range"=>{v["pattern"]=json!(if title.contains("선택"){"XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"}else if title.contains("확대"){"XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"}else{"XXX\nX@X\nXXX"});v["includeAnchor"]=json!(true);},
  "skill-seconds"=>v["value"]=json!(if title.contains("공통 수명"){3}else if title.contains("온기"){2}else{1}),
  "skill-healing-amount"=>v["value"]=json!(100),"skill-pair-transfer"=>v["percent"]=json!(50),"skill-attack-speed-buff"=>v["points"]=json!(20),
  "skill-modifier-buff"|"skill-modifier-healing"=>v["percent"]=json!(50),
  "skill-attribute-effect"=>{v["attribute"]=json!("attack");v["operation"]=json!("percent");v["amount"]=json!(10);},_=>()
 }}doc
}
#[test]
fn paired_areas_graph_roundtrip_preserves_pair_selection_settlement_and_conditional_replacement(){
 let p=compile(&paired_areas_fixture().to_string(),"pair".into()).unwrap();let q=PortableGraph::from_ron(&p.to_ron().unwrap()).unwrap();assert_eq!(p.operation(),q.operation());let GraphOperation::ResonancePairedAreas{program}=q.operation()else{panic!()};assert_eq!(program.healing,100);assert_eq!(program.duration_ms,3000);assert_eq!(program.interval_ms,1000);assert_eq!(program.modifiers.len(),6);
 let native=q.load_native().unwrap();assert_eq!(native.skills[0].cast_selection(),crate::game::ability::SkillCastSelection::PlayerTilePair);
}
#[test]
fn paired_areas_graph_rejects_wrong_pulse_result_bad_duration_and_selection_modifier(){
 for case in 0..5 {let mut d=paired_areas_fixture();let nodes=d["nodes"].as_array_mut().unwrap();
  match case {0=>nodes.iter_mut().find(|n|n["kind"]=="skill-pair-transfer").unwrap()["values"]["percent"]=json!(101),1=>nodes.iter_mut().find(|n|n["title"].as_str().unwrap().contains("첫 회복")).unwrap()["values"]["value"]=json!(0),2=>nodes.iter_mut().find(|n|n["kind"]=="skill-target-state").unwrap()["values"]["sourceExit"]=json!("end"),3=>{let heal=nodes.iter().find(|n|n["kind"]=="skill-pair-base-heal").unwrap()["id"].clone();d["edges"].as_array_mut().unwrap().retain(|e|!(e["source"]["node"]==heal&&e["source"]["port"]=="result"));},_=>{let range=nodes.iter().find(|n|n["title"].as_str().unwrap().contains("선택 가능")).unwrap()["id"].clone();let modifier=nodes.iter().find(|n|n["kind"]=="skill-modifier-range").unwrap()["id"].clone();for e in d["edges"].as_array_mut().unwrap(){if e["target"]["node"]==modifier&&e["target"]["port"]=="target"{e["source"]["node"]=range.clone();}}}}
  assert!(compile(&d.to_string(),"bad".into()).is_err(),"{case}");
 }
}
#[test]
fn paired_areas_native_rejects_single_tile_or_owned_installation(){
 let native=compile(&paired_areas_fixture().to_string(),"pair".into()).unwrap().load_native().unwrap();for case in 0..3{let mut db=native.clone();let s=&mut db.skills[0];match case{0=>{let crate::game::ability::SkillCastTargetingDef::Explicit{selection,..}=&mut s.cast_targeting;*selection=crate::game::ability::SkillCastSelection::PlayerTile;},1=>s.steps[0].delay_ms=1,_=>s.use_lifetime=crate::game::ability::SkillUseLifetime::ActionAndEffects{effects:vec![crate::game::ability::SkillUseEffectRef{step:"finish".into(),effect:"pair".into()}]}}assert!(crate::game::data::skill_data::SkillDatabase::from_ron("bad",&ron::ser::to_string(&db).unwrap()).is_err());}
}

pub(crate) fn fusion_placement_fixture(bloom:bool)->serde_json::Value{
 let raw=if bloom{include_str!("../../../../content_assembler/content/laetitia-bloom.node-graph.json")}else{include_str!("../../../../content_assembler/content/laetitia.node-graph.json")};
 let mut v:serde_json::Value=serde_json::from_str(raw).unwrap();
 for n in v["nodes"].as_array_mut().unwrap(){let kind=n["kind"].as_str().unwrap().to_owned();let title=n["title"].as_str().unwrap().to_owned();let p=n["values"].as_object_mut().unwrap();
  match kind.as_str(){
   "skill-resonance-amount"=>{p.insert("value".into(),json!(100));},
   "skill-range"=>{let pattern=if title.contains("선택"){"XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"}else if title.contains("설치 위치"){".X.\nX@X\n.X."}else if title.contains("작은 친구"){"@"}else if title.contains("확대"){"XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"}else{"XXX\nX@X\nXXX"};p.insert("pattern".into(),json!(pattern));p.insert("includeAnchor".into(),json!(true));},
   "skill-seconds" if title.contains("대기")=>{p.insert("value".into(),json!(5));},
   "skill-seconds" if title.contains("도약")=>{p.insert("value".into(),json!(0.1));},
   "skill-weapon-power"=>{p.insert("coefficient".into(),json!(1));},
   "skill-damage-bonus"=>{p.insert("bonus".into(),json!(100));},
   "skill-fusion-count-power"|"skill-modifier-mobile-power"|"skill-modifier-mobile-vulnerability"=>{p.insert("percent".into(),json!(50));},
   "skill-fusion-family"=>{p.insert("maxUnits".into(),json!(20));p.insert("maxTiles".into(),json!(8));},
   "skill-mobile-effect"=>{p.insert("speed".into(),json!(10));},_=>()
  }
 }v
}
#[test]
fn fusion_placement_compiler_both_forms_preserve_tiers_power_and_modifiers(){
 for bloom in [false,true]{let p=compile(&fusion_placement_fixture(bloom).to_string(),"fusion".into()).unwrap();let q=PortableGraph::from_ron(&p.to_ron().unwrap()).unwrap();assert_eq!(p.operation(),q.operation());let GraphOperation::ResonanceFusionPlacement{program}=q.operation()else{panic!()};assert_eq!(program.primary,if bloom{2}else{1});assert_eq!(program.adjacent,1);assert_eq!(program.tiers.iter().map(|t|t.minimum).collect::<Vec<_>>(),[1,2,3]);assert_eq!(program.tiers[2].profile.power.resolve(100,5).unwrap(),500);assert_eq!(program.tiers[2].profile.range.rows.len(),7);assert_eq!(program.tiers[2].profile.modifiers.len(),3);assert_eq!(program.seed.as_ref().unwrap().duration_ms,10000);}
}
#[test]
fn fusion_placement_compiler_unresolved_and_duplicate_tiers_rejected(){
 assert!(compile(include_str!("../../../../content_assembler/content/laetitia.node-graph.json"),"fusion".into()).is_err());
 let mut v=fusion_placement_fixture(false);let n=v["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-fusion-tier"&&n["values"]["minimum"]==3).unwrap();n["values"]["minimum"]=json!(2);assert!(compile(&v.to_string(),"fusion".into()).is_err());
}
#[test]
fn fusion_placement_compiler_rejects_unbound_modifier_target(){
 let mut v=fusion_placement_fixture(false);let selection=v["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-range"&&n["title"].as_str().unwrap().contains("선택")).unwrap()["id"].as_str().unwrap().to_owned();let modifier=v["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-modifier-range").unwrap()["id"].as_str().unwrap().to_owned();let edge=v["edges"].as_array_mut().unwrap().iter_mut().find(|e|e["target"]["node"]==modifier&&e["target"]["port"]=="target").unwrap();edge["source"]["node"]=json!(selection);assert!(compile(&v.to_string(),"fusion".into()).is_err());
}

pub(crate) fn status_strike_fixture()->serde_json::Value{
 let mut v:serde_json::Value=serde_json::from_str(include_str!("../../../../content_assembler/content/snow-queen.node-graph.json")).unwrap();
 for n in v["nodes"].as_array_mut().unwrap(){let kind=n["kind"].as_str().unwrap().to_owned();let title=n["title"].as_str().unwrap().to_owned();let p=n["values"].as_object_mut().unwrap();match kind.as_str(){
  "skill-resonance-amount"=>{p.insert("value".into(),json!(100));},
  "skill-count-positive"=>{p.insert("value".into(),json!(8));},
  "skill-range"=>{let pattern=if title.contains("T자"){"XXX\n.X.\n.@."}else if title.contains("탐색"){"XXXXXXX\nXXXXXXX\nXXXXXXX\nXXX@XXX\nXXXXXXX\nXXXXXXX\nXXXXXXX"}else if title.contains("확대"){"XXXXX\nXXXXX\nXX@XX\nXXXXX\nXXXXX"}else{"@"};p.insert("pattern".into(),json!(pattern));p.insert("includeAnchor".into(),json!(!title.contains("T자")));},
  "skill-seconds" if title.contains("지연")=>{p.insert("value".into(),json!(0.2));},
  "skill-weapon-power"=>{p.insert("coefficient".into(),json!(1));},
  "skill-mobile-effect"=>{p.insert("speed".into(),json!(5));},
  "skill-modifier-control-vulnerability"|"skill-modifier-mobile-power"|"skill-modifier-mobile-vulnerability"=>{p.insert("percent".into(),json!(50));},_=>()
 }}v
}
#[test]
fn status_strike_compiler_preserves_transition_mobile_and_modifier_bindings_through_ron(){let p=compile(&status_strike_fixture().to_string(),"status".into()).unwrap();let q=PortableGraph::from_ron(&p.to_ron().unwrap()).unwrap();assert_eq!(p.operation(),q.operation());let GraphOperation::ResonanceStatusStrike{program}=q.operation()else{panic!()};assert_eq!(program.from.duration_ms,3000);assert_eq!(program.to.duration_ms,2000);assert_eq!(program.to.modifiers.len(),2);assert_eq!(program.mobile.modifiers.len(),3);assert_eq!(program.mobile.delay_ms,200);assert_eq!(program.limit,8);assert_eq!(program.death.as_ref().unwrap().states.len(),2);}
#[test]
fn status_strike_compiler_rejects_unresolved_and_cross_bound_death_sources(){assert!(compile(include_str!("../../../../content_assembler/content/snow-queen.node-graph.json"),"status".into()).is_err());let mut v=status_strike_fixture();let id=v["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-status-death-mobile").unwrap()["id"].as_str().unwrap().to_owned();let edge=v["edges"].as_array_mut().unwrap().iter_mut().find(|e|e["target"]["node"]==id&&e["target"]["port"]=="limit").unwrap();edge["source"]["port"]=json!("invalid");assert!(compile(&v.to_string(),"status".into()).is_err());}

#[test]
fn guard_charge_compiler_preserves_explicit_relic_meter_and_rejects_invalid_limits() {
    let doc=parry_fixture();
    let graph=compile(&doc.to_string(),"guard".into()).unwrap();
    let reloaded=PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();
    let GraphOperation::ResonanceParry {program,..}=reloaded.operation() else {panic!()};
    let charge=program.charge.as_ref().unwrap();
    assert_eq!((charge.capacity,charge.primary.numerator,charge.splash.numerator,charge.splash.denominator),(1000,2,1,2));
    assert_eq!(charge.relic_definition,"awakened-black-swan");
    assert_eq!(graph.operation(),reloaded.operation());
    assert!(PortableGraph::from_ron(&graph.to_ron().unwrap().replace("capacity: 1000","capacity: 0")).is_err());
    let mut wrong=doc;
    let condition=wrong["nodes"].as_array_mut().unwrap().iter_mut().find(|n|n["kind"]=="skill-equipment-condition").unwrap();
    condition["values"]["variant"]=json!("corrupt");
    assert!(compile(&wrong.to_string(),"guard".into()).is_err());
}
#[test]
fn support_modifier_targets_are_explicit_and_do_not_accept_selection_range_expansion() {
    let mut doc=support_fixture(true);
    let plan=compile(&doc.to_string(),"support".into()).unwrap();
    let GraphOperation::ResonanceSupport {program,..}=plan.operation() else {panic!()};
    assert_eq!(program.policy.modifiers.len(),2);
    let single=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-modifier-single-support").unwrap()["id"].clone();
    let range_modifier=doc["nodes"].as_array().unwrap().iter().find(|n|n["kind"]=="skill-modifier-range").unwrap()["id"].clone();
    let selection=doc["edges"].as_array().unwrap().iter().find(|e|e["target"]["node"]==single&&e["target"]["port"]=="range").unwrap()["source"].clone();
    let edge=doc["edges"].as_array_mut().unwrap().iter_mut().find(|e|e["target"]["node"]==range_modifier&&e["target"]["port"]=="target").unwrap();
    edge["source"]=selection;
    assert!(compile(&doc.to_string(),"support".into()).is_err());
}
