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
                remaining_cost: limit,
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
        let _ = (input, expected);
        Err(active_run_checkpoint_codec_error("V8 conversion", "legacy lifecycle has no exact ownership/instance mapping; original preserved"))
    }
}
