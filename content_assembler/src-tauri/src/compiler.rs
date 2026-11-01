use serde_json::Value;

pub fn compile(input: Value) -> Result<Value, String> {
    let request = serde_json::from_value(input).map_err(|error| error.to_string())?;
    let response = game_core::authoring::node_graph::compile_request(request)?;
    serde_json::to_value(response).map_err(|error| error.to_string())
}

pub fn compile_composition(input: Value) -> Result<Value, String> {
    let request = serde_json::from_value(input).map_err(|error| error.to_string())?;
    let response = game_core::authoring::composition::compile_request(request)?;
    serde_json::to_value(response).map_err(|error| error.to_string())
}
pub fn run_composition(input: Value) -> Result<Value, String> {
    let request = serde_json::from_value(input).map_err(|error| error.to_string())?;
    let response = game_core::authoring::composition::run_composition(
        request,
        &game_core::authoring::CancellationToken::default(),
    )?;
    serde_json::to_value(response).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn composition_bridges_keep_compiler_identity_and_reject_a_foreign_execution_definition() {
        let source = include_str!(
            "../../../core_enhanced/tests/fixtures/skill_composition/woodsman_monk.skill.yaml"
        );
        let compiled = compile_composition(serde_json::json!({
            "source": source, "epoch": 1, "revision": 2, "request_id": 3,
        }))
        .unwrap();
        assert_eq!(compiled["status"], "compiled");
        assert_eq!(compiled["revision"], 2);
        let rejected = run_composition(serde_json::json!({
            "source": source, "epoch": 1, "revision": 2, "request_id": 4,
            "artifact_digest": "foreign-definition", "root": "no-content-root",
            "selection": {"packs":["official"],"winners":[]}, "scenario":"{}",
        }))
        .unwrap();
        assert_eq!(rejected["status"], "rejected");
        assert_eq!(rejected["failure"]["stage"], "definition");
        assert!(rejected["identity"].is_null());
    }
    #[test]
    fn native_bridge_returns_core_diagnostics_without_success_payload() {
        let result = compile(serde_json::json!({
            "source": "{}", "skill_id": "bridge-test", "epoch": 3,
            "revision": 8, "request_id": 12,
        }))
        .unwrap();
        assert_eq!(result["status"], "rejected");
        assert_eq!(result["epoch"], 3);
        assert_eq!(result["revision"], 8);
        assert_eq!(result["request_id"], 12);
        assert!(!result["diagnostics"].as_array().unwrap().is_empty());
        assert!(result.get("ron").is_none());
    }
}
