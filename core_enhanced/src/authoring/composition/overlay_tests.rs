use super::*;
use serde_json::{json, Value as Json};

pub(crate) fn module(native: &crate::game::data::skill_data::SkillDatabase) -> Json {
    let mut d = super::owned_action_tests::module(native);
    for (i, skill) in native.skills.iter().enumerate() {
        let effects: Vec<_> = skill.steps[0]
            .effects
            .iter()
            .map(|effect| match &effect.op {
                crate::game::ability::SkillEffectDef::DamageExpr {
                    amount,
                    damage_type,
                } => {
                    json!({"Damage":{"id":effect.id,"tags":effect.tags,
                        "amount":{"Literal":{"Amount":amount}},"damage_type":damage_type}})
                }
                _ => json!({"Append":{"effects":{"Literal":{"Effects":[effect]}}}}),
            })
            .collect();
        d["skills"][i]["body"]["Action"]["effects"] = json!(effects);
    }
    d
}

#[test]
fn overlay_composition_preserves_profile_child_expression_and_use_ownership() {
    let original = crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::attack_overlay_fixture().to_string(),
        "r11_aura".into(),
    )
    .unwrap()
    .load_native()
    .unwrap();
    let d = module(&original);
    let artifact = compile_json(&d.to_string()).unwrap();
    let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    let native = artifact.load_native().unwrap();
    for (old, new) in original.skills.iter().zip(&native.skills) {
        assert_eq!(old.resonance_admission, new.resonance_admission);
        assert_eq!(old.focus_permissions, new.focus_permissions);
        assert_eq!(old.cast_targeting, new.cast_targeting);
        assert_eq!(
            serde_json::to_value(&old.steps[0].effects).unwrap(),
            serde_json::to_value(&new.steps[0].effects).unwrap()
        );
    }
    let parent = &native.skills[0];
    assert!(parent
        .use_lifetime
        .owns_effect(&parent.steps[0].id, &parent.steps[0].effects[0].id));
    assert!(native.skills[1].flow.is_none());
    assert!(artifact
        .sources
        .keys()
        .any(|p| p.starts_with("skills[1].steps[0].effects[0]")));
    if let Some(root) = std::env::var_os("OVERLAY_SOURCE_EXPORT_DIR") {
        use std::io::Write;
        let path = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join("overlay.technical.json"))
            .unwrap()
            .write_all(serde_json::to_string_pretty(&d).unwrap().as_bytes())
            .unwrap();
    }
}

#[test]
fn overlay_composition_rejects_async_cost_missing_child_and_unused_invalid_overlay() {
    let original = crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::attack_overlay_fixture().to_string(),
        "r11_aura".into(),
    )
    .unwrap()
    .load_native()
    .unwrap();
    let d = module(&original);
    for fault in [
        "prepare", "cost", "wait", "sequence", "missing", "self", "duration", "unused",
    ] {
        let mut invalid = d.clone();
        match fault {
            "prepare" => invalid["skills"][1]["prepare_ms"] = json!(1),
            "cost" => invalid["skills"][1]["admission"] = d["skills"][0]["admission"].clone(),
            "wait" => invalid["skills"][1]["body"]["Action"]["wait"] = json!("EffectsSettled"),
            "sequence" => {
                let body = invalid["skills"][1]["body"].take();
                invalid["skills"][1]["body"] = json!({"Sequence":{"id":"extra","children":[body]}});
            }
            "missing" => {
                invalid["skills"].as_array_mut().unwrap().pop();
            }
            "self" => {
                invalid["skills"][0]["body"]["Action"]["effects"][0]["Append"]["effects"]
                    ["Literal"]["Effects"][0]["op"]["Sustained"]["definition"]
                    ["BasicAttackOverlay"]["goodbye_skill"] = json!("r11_aura")
            }
            _ => {
                let value = &mut invalid["skills"][0]["body"]["Action"]["effects"][0]["Append"]
                    ["effects"]["Literal"];
                value["Effects"][0]["op"]["Sustained"]["definition"]["BasicAttackOverlay"]
                    ["duration_ms"] = json!(0);
                if fault == "unused" {
                    let value = value.clone();
                    invalid = d.clone();
                    invalid["values"] = json!([{"name":"unused-invalid-overlay","value":value}]);
                }
            }
        }
        let error = compile_json(&invalid.to_string()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Native, "{fault}: {error:?}");
        if fault == "unused" {
            assert_eq!(
                error.source.definition,
                "values[0].value.Effects[0].op.Sustained.definition.BasicAttackOverlay"
            );
        }
    }
}
