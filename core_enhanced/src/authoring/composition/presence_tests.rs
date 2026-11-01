use super::*;
use serde_json::json;

#[test]
fn presence_append_preserves_intrinsic_and_rejects_invalid_immediate_structure() {
    for form in ["presence"] {
        let content = crate::authoring::node_graph::compile(
            &crate::authoring::node_graph::tests::presence_fixture().to_string(),
            "r11_aura".into(),
        )
        .unwrap()
        .load_content()
        .unwrap();
        let document = super::owned_action_tests::module(&content.skills);
        let artifact = compile_json(&document.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let loaded = artifact.load_native().unwrap();
        let (old, new) = (&content.skills.skills[0], &loaded.skills[0]);
        assert_eq!(new.resonance_admission, old.resonance_admission);
        assert_eq!(new.cast_targeting, old.cast_targeting);
        assert_eq!(
            new.use_lifetime,
            crate::game::ability::SkillUseLifetime::Action
        );
        assert_eq!(
            serde_json::to_value(&new.steps[0].effects).unwrap(),
            serde_json::to_value(&old.steps[0].effects).unwrap()
        );
        // The intrinsic is owned by the equipped fragment program itself.
        // There is no second weapon/catalog subscription authority.
        assert!(artifact.equipment_attacks().is_empty());
        assert!(content.mark_weapons.is_empty());
        for fault in ["prepare", "completion", "target", "repeat"] {
            let mut invalid = document.clone();
            match fault {
                "prepare" => invalid["skills"][0]["prepare_ms"] = json!(1),
                "completion" => {
                    invalid["skills"][0]["body"]["Action"]["wait"] = json!("EffectsSettled")
                }
                "target" => {
                    invalid["skills"][0]["body"]["Action"]["target"]["Literal"]["Target"] =
                        json!("CastTarget")
                }
                _ => {
                    let body = invalid["skills"][0]["body"].take();
                    invalid["skills"][0]["body"] = json!({"Repeat":{"id":"repeat","count":{"Literal":{"Count":2}},"interval":{"Literal":{"Milliseconds":1}},"body":body}});
                }
            }
            let error = compile_json(&invalid.to_string()).unwrap_err();
            assert_eq!(
                error.code,
                DiagnosticCode::Native,
                "{form}/{fault}: {error:?}"
            );
        }
        let mut invalid = document.clone();
        let mut effect = invalid["skills"][0]["body"]["Action"]["effects"][0]["Append"]["effects"]
            ["Literal"]
            .clone();
        effect["Effects"][0]["op"]["PresenceBurst"]["program"]["marks"]["periodic_interval_ms"] =
            json!(0);
        invalid["values"] = json!([{"name":"unused-invalid-presence","value":effect}]);
        let error = compile_json(&invalid.to_string()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Native);
        assert_eq!(
            error.source.definition,
            "values[0].value.Effects[0].op.PresenceBurst.program"
        );
        if let Some(root) = std::env::var_os("PRESENCE_SOURCE_EXPORT_DIR") {
            let root = std::path::PathBuf::from(root);
            std::fs::create_dir_all(&root).unwrap();
            use std::io::Write;
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("presence.technical.json"))
                .unwrap()
                .write_all(serde_json::to_string_pretty(&document).unwrap().as_bytes())
                .unwrap();
        }
    }
}
