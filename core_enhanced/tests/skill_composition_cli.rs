use game_core::authoring::composition::Artifact;
use std::{
    io::Write,
    process::{Command, Stdio},
};

const SOURCE: &str = include_str!("fixtures/skill_composition/woodsman_monk.json");

fn compile(input: &str) -> std::process::Output {
    compile_bytes(input.as_bytes())
}
fn compile_bytes(input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_compile_skill_composition"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn composition_cli_input_encoding_and_byte_limits_return_structured_diagnostics() {
    for (input, code) in [
        (vec![0xff], "Parse"),
        (
            vec![b' '; game_core::authoring::composition::MAX_INPUT_BYTES + 1],
            "Limit",
        ),
    ] {
        let output = compile_bytes(&input);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(diagnostic["code"], code);
        assert_eq!(diagnostic["source"]["definition"], "$");
        assert_eq!(diagnostic["source"]["calls"], serde_json::json!([]));
    }
}

#[test]
fn composition_cli_produces_a_saved_reloadable_artifact() {
    let output = compile(SOURCE);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let path = std::env::temp_dir().join(format!("composition-{}.ron", uuid::Uuid::new_v4()));
    std::fs::write(&path, &output.stdout).unwrap();
    let saved = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let loaded = Artifact::from_ron(&saved).unwrap();
    let database = loaded.load_native().unwrap();
    assert_eq!(database.skills.len(), 2);
    assert_eq!(database.skills[0].steps.len(), 3);
    assert_eq!(database.skills[1].steps.len(), 1);
}

#[test]
fn composition_cli_rejects_with_structured_diagnostic_and_no_partial_artifact() {
    let invalid = SOURCE.replace(
        "\"Shared\": \"burst-power\"",
        "\"Shared\": \"missing-power\"",
    );
    assert_ne!(invalid, SOURCE);
    let output = compile(&invalid);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(diagnostic["code"], "Reference");
    assert!(diagnostic["source"]["definition"]
        .as_str()
        .unwrap()
        .contains("skills[1]"));
    assert!(diagnostic["source"]["calls"].as_array().unwrap().len() >= 2);
}
