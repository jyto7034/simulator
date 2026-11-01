use super::*;
use crate::game::data::skill_data::SkillDatabase;
use serde_json::{json, Value as Json};

// A test-only bridge: original graph fixtures remain explicitly technical.
// This neither publishes canonical content nor supplies missing balance values.
pub(crate) fn module(native: &SkillDatabase) -> Json {
    let skills: Vec<_> = native.skills.iter().map(|skill| {
        assert_eq!(skill.steps.len(), 1);
        let step = &skill.steps[0];
        json!({
            "id": skill.id, "name": skill.name, "kind": skill.kind,
            "cast_targeting": skill.cast_targeting, "prepare_ms": skill.focus_time_ms,
            "focus_permissions": skill.focus_permissions, "admission": skill.resonance_admission,
            "body": {"Action": {"id": "entry",
                "target": {"Literal": {"Target": step.target}},
                "delivery": {"Literal": {"Delivery": step.delivery}},
                "effects": [{"Append": {"effects": {"Literal": {"Effects": step.effects}}}}],
                "wait": "Released"
            }}
        })
    }).collect();
    json!({"version": 1, "id": "owned-native-action-fixture", "values": [],
        "definitions": [], "skills": skills})
}

pub(crate) fn content(graph: Json) -> crate::authoring::node_graph::NativeGraphContent {
    let original = crate::authoring::node_graph::compile(&graph.to_string(), "r11_aura".into())
        .unwrap()
        .load_content()
        .unwrap();
    let authored = module(&original.skills);
    let artifact = compile_json(&authored.to_string()).unwrap();
    crate::authoring::node_graph::NativeGraphContent {
        skills: artifact.load_native().unwrap(),
        ..original
    }
}

#[test]
fn owned_native_actions_preserve_programs_admission_and_effect_lifetimes() {
    use crate::authoring::node_graph::{compile, tests::*};
    for (name, graph) in [
        ("relation-base", relation_action_fixture(false)),
        ("relation-bloom", relation_action_fixture(true)),
        ("counter-base", counter_channel_fixture(false)),
        ("counter-bloom", counter_channel_fixture(true)),
        ("input-plan", input_plan_fixture()),
        ("prepared-volley", prepared_volley_fixture()),
    ] {
        let native = compile(&graph.to_string(), "r11_aura".into())
            .unwrap()
            .load_native()
            .unwrap();
        let document = module(&native);
        let artifact = compile_json(&document.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let loaded = artifact.load_native().unwrap();
        assert_eq!(loaded.skills.len(), native.skills.len());
        for (old, new) in native.skills.iter().zip(&loaded.skills) {
            assert_eq!(new.id, old.id);
            assert_eq!(new.kind, old.kind);
            assert_eq!(new.cast_targeting, old.cast_targeting);
            assert_eq!(new.resonance_admission, old.resonance_admission);
            assert_eq!(new.focus_permissions, old.focus_permissions);
            assert_eq!(
                serde_json::to_value(&new.steps[0].effects).unwrap(),
                serde_json::to_value(&old.steps[0].effects).unwrap()
            );
            let step = &new.steps[0];
            assert!(new.use_lifetime.owns_effect(&step.id, &step.effects[0].id));
        }
        // Reject known contracts after successful parsing, rather than letting
        // the compiler silently rewrite an incompatible authored declaration.
        for fault in ["admission", "preparation", "completion"] {
            let mut invalid = document.clone();
            match fault {
                "admission" => invalid["skills"][0]["admission"]["capacity"] = json!(99),
                "preparation" => invalid["skills"][0]["prepare_ms"] = json!(1),
                _ => invalid["skills"][0]["body"]["Action"]["wait"] = json!("EffectsSettled"),
            }
            let diagnostic = compile_json(&invalid.to_string()).unwrap_err();
            assert_eq!(
                diagnostic.code,
                DiagnosticCode::Native,
                "{name}/{fault}: {diagnostic:?}"
            );
        }
        let mut invalid = document.clone();
        let mut effect = invalid["skills"][0]["body"]["Action"]["effects"][0]["Append"]["effects"]
            ["Literal"]
            .clone();
        let operation = effect["Effects"][0]["op"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap();
        operation["program"]["definition"] = json!("");
        invalid["values"] = json!([{ "name": "unused-invalid-program", "value": effect }]);
        let diagnostic = compile_json(&invalid.to_string()).unwrap_err();
        assert_eq!(
            diagnostic.code,
            DiagnosticCode::Native,
            "{name}: {diagnostic:?}"
        );
        assert!(diagnostic
            .source
            .definition
            .starts_with("values[0].value.Effects[0].op."));
        if let Some(root) = std::env::var_os("OWNED_ACTION_EXPORT_DIR") {
            let root = std::path::PathBuf::from(root);
            std::fs::create_dir_all(&root).unwrap();
            let path = root.join(format!("{name}.technical.json"));
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
                .unwrap();
            file.write_all(serde_json::to_string_pretty(&document).unwrap().as_bytes())
                .unwrap();
        }
    }
}
