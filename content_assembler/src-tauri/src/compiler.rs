use serde_json::Value;

pub fn compile(input: Value) -> Result<Value, String> {
    let request = serde_json::from_value(input).map_err(|error| error.to_string())?;
    let response = game_core::authoring::node_graph::compile_request(request)?;
    serde_json::to_value(response).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
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
