//! Explicit original-checksum/content-identity checked V8 -> V12 conversion.
use super::*;
use crate::game::{data::content_pack::ContentIdentity, employee::BreakdownCounter};

pub(super) fn insert_breakdown_counters(
    checkpoint: &mut serde_json::Map<String, Value>,
    limit: u32,
) -> Result<(), GameError> {
    let bad = |message: &str| active_run_checkpoint_codec_error("Breakdown conversion", message);
    let employees = checkpoint
        .get_mut("roster")
        .and_then(|v| v.get_mut("employees"))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| bad("missing employee roster"))?;
    for employee in employees.values_mut() {
        let object = employee
            .as_object_mut()
            .ok_or_else(|| bad("invalid employee"))?;
        if object.contains_key("breakdown") {
            return Err(bad("legacy employee contains new counter"));
        }
        let condition = object
            .get("condition")
            .ok_or_else(|| bad("missing condition"))?;
        let trauma = condition
            .get("trauma")
            .and_then(Value::as_u64)
            .ok_or_else(|| bad("invalid trauma"))?;
        let physical = condition
            .get("physical_damage")
            .and_then(Value::as_u64)
            .ok_or_else(|| bad("invalid physical damage"))?;
        if trauma > 100 || physical > 6 {
            return Err(bad("condition out of range"));
        }
        let counter = if trauma == 100 && physical < 6 {
            BreakdownCounter::Active {
                remaining_nodes: limit,
                entered_node: None,
            }
        } else {
            BreakdownCounter::Inactive
        };
        object.insert(
            "breakdown".into(),
            serde_json::to_value(counter).map_err(|e| bad(&e.to_string()))?,
        );
    }
    Ok(())
}

impl GameCore {
    pub(crate) fn convert_active_run_v8(
        &self,
        input: &[u8],
        expected: &ContentIdentity,
    ) -> Result<Vec<u8>, GameError> {
        let bad = |message: &str| active_run_checkpoint_codec_error("V8 conversion", message);
        if input.len() > 32 * 1024 * 1024 {
            return Err(bad("payload exceeds 32 MiB"));
        }
        let v7::UniqueJson(mut raw) =
            serde_json::from_slice(input).map_err(|e| bad(&e.to_string()))?;
        if raw.as_object().is_none_or(|o| o.len() != 3) || raw["version"] != 8 {
            return Err(bad("expected exact V8 envelope"));
        }
        let checkpoint = raw
            .get("checkpoint")
            .ok_or_else(|| bad("missing checkpoint"))?;
        let mut bytes = Vec::new();
        write_canonical_json(checkpoint, &mut bytes).map_err(|e| bad(&e.to_string()))?;
        let digest = format!("{:x}", Sha256::digest(bytes));
        if raw["checkpoint_sha256"].as_str() != Some(&digest) {
            return Err(bad("original checksum mismatch"));
        }
        let identity: ContentIdentity =
            serde_json::from_value(checkpoint["content_identity"].clone())
                .map_err(|e| bad(&e.to_string()))?;
        if &identity != expected {
            return Err(bad("original content identity mismatch"));
        }
        let checkpoint = raw
            .get_mut("checkpoint")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| bad("invalid checkpoint"))?;
        insert_breakdown_counters(
            checkpoint,
            self.run_policy().corrosion.breakdown_collapse_node_limit,
        )?;
        checkpoint.insert(
            "content_identity".into(),
            serde_json::to_value(self.game_data.identity()).map_err(|e| bad(&e.to_string()))?,
        );
        super::convert_legacy_employee_resonance(checkpoint)?;
        let candidate: RunCheckpointPayload =
            serde_json::from_value(Value::Object(checkpoint.clone()))
                .map_err(|e| bad(&e.to_string()))?;
        self.validate_run_checkpoint_payload_for_continue(&candidate)?;
        encode_active_run_checkpoint(&candidate)
    }
}
