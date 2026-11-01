//! Bounded references to earlier public responses. This is a local driver,
//! never a Core authority, saved policy, or gameplay state owner.
use super::{invalid, ByteBuffer, Error, StepOutcome};
use game_core::{
    application::*,
    authoring::{Assertion, JsonValueType, ResultPointer, TrialStep},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::VecDeque;

pub(super) const HISTORY_CAP: usize = 256;
pub(super) const HISTORY_BYTE_CAP: usize = 1024 * 1024;
const BINDING_CAP: usize = 64;
const POINTER_BYTE_CAP: usize = 256;

#[derive(Default)]
pub(super) struct History {
    values: VecDeque<(usize, Value, usize)>,
    bytes: usize,
    pub dropped: usize,
}
impl History {
    pub fn usage(&self) -> (usize, usize, usize) {
        (self.values.len(), self.bytes, self.dropped)
    }
    pub fn push(&mut self, index: usize, value: Option<Value>, bytes: usize) {
        let Some(value) = value else {
            self.dropped += 1;
            return;
        };
        if bytes > HISTORY_BYTE_CAP {
            self.dropped += 1;
            return;
        }
        while self.values.len() == HISTORY_CAP || self.bytes + bytes > HISTORY_BYTE_CAP {
            if let Some((_, _, bytes)) = self.values.pop_front() {
                self.bytes -= bytes;
                self.dropped += 1;
            }
        }
        self.bytes += bytes;
        self.values.push_back((index, value, bytes));
    }
    pub fn get(&self, from: &ResultPointer) -> Result<&Value, Error> {
        self.document(from.step)?
            .pointer(&from.pointer)
            .ok_or_else(|| invalid("public response pointer is absent").into())
    }
    fn document(&self, step: usize) -> Result<&Value, Error> {
        self.values
            .iter()
            .find(|(index, _, _)| *index == step)
            .map(|(_, value, _)| value)
            .ok_or_else(|| {
                invalid("public response reference is absent, missing or retired").into()
            })
    }
}

fn pointer(value: &str) -> bool {
    value.len() <= POINTER_BYTE_CAP && (value.is_empty() || value.starts_with('/'))
}
pub(super) fn validate(step: &TrialStep, index: usize) -> Result<usize, Error> {
    match step {
        TrialStep::Command {
            command,
            context,
            bindings,
        } => {
            if !command.is_object() || !context.is_object() || bindings.len() > BINDING_CAP {
                return Err(invalid("invalid bounded scripted command").into());
            }
            let kind = command
                .get("type")
                .ok_or_else(|| invalid("script command type is missing"))?;
            serde_json::from_value::<ApplicationCommandKind>(kind.clone())?;
            for binding in bindings {
                if binding.from.step >= index
                    || !pointer(&binding.from.pointer)
                    || !pointer(&binding.to)
                    || !(binding.to.starts_with("/command/") || binding.to.starts_with("/context/"))
                {
                    return Err(invalid("script references must target an earlier public response and command/context field").into());
                }
            }
            if bindings.is_empty() {
                ApplicationCommand::from_document(command.clone().into())?;
            }
            Ok(1)
        }
        TrialStep::Assert { from, .. } => {
            if from.step >= index || !pointer(&from.pointer) {
                return Err(invalid("invalid bounded public assertion reference").into());
            }
            Ok(0)
        }
        _ => Ok(1),
    }
}
fn matches(value: &Value, kind: JsonValueType) -> bool {
    match kind {
        JsonValueType::Null => value.is_null(),
        JsonValueType::Bool => value.is_boolean(),
        JsonValueType::Number => value.is_number(),
        JsonValueType::String => value.is_string(),
        JsonValueType::Array => value.is_array(),
        JsonValueType::Object => value.is_object(),
    }
}
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Context {
    source_command_id: Option<String>,
    battle_event_after_seq: u64,
    expected_battle_authority: Option<ApplicationBattleAuthority>,
}
pub(super) fn resolve(
    step: &TrialStep,
    history: &History,
) -> Result<(ApplicationCommand, ApplicationExecutionContext, Value), Error> {
    let TrialStep::Command {
        command,
        context,
        bindings,
    } = step
    else {
        return Err(invalid("expected scripted command").into());
    };
    let mut document = json!({"command":command,"context":context});
    for binding in bindings {
        let source = history.get(&binding.from)?;
        if !matches(source, binding.value_type) {
            return Err(invalid("public response binding type mismatch").into());
        }
        let destination = document
            .pointer_mut(&binding.to)
            .ok_or_else(|| invalid("script target field does not exist"))?;
        *destination = source.clone();
    }
    // Bound the resolved input before public execution. Response retention and
    // export are separately bounded and cannot alter already-executed gameplay.
    let mut encoded = ByteBuffer {
        bytes: Vec::new(),
        cap: HISTORY_BYTE_CAP,
    };
    serde_json::to_writer(&mut encoded, &document)?;
    let command = ApplicationCommand::from_document(document["command"].clone().into())?;
    let context: Context = serde_json::from_value(document["context"].clone())?;
    if context
        .source_command_id
        .as_ref()
        .is_some_and(|s| s.len() > 256)
    {
        return Err(invalid("source command metadata cap exceeded").into());
    }
    Ok((
        command,
        ApplicationExecutionContext {
            source_command_id: context.source_command_id,
            battle_event_after_seq: context.battle_event_after_seq,
            expected_battle_authority: context.expected_battle_authority,
        },
        document,
    ))
}
pub(super) fn assertion(
    from: &ResultPointer,
    assertion: &Assertion,
    history: &History,
) -> Result<StepOutcome, Error> {
    // A missing/retired response makes the assertion unavailable. Only a
    // missing pointer inside a retained response can satisfy Exists(false).
    let value = history.document(from.step)?.pointer(&from.pointer);
    let accepted = match assertion {
        Assertion::Exists { expected } => value.is_some() == *expected,
        Assertion::Type { value_type } => value.is_some_and(|v| matches(v, *value_type)),
        Assertion::Equals { value: expected } => value == Some(expected),
    };
    if !accepted {
        return Ok(super::driver_outcome("public scenario assertion failed", 0));
    }
    let mut outcome = super::result::<bool>(Ok(true));
    outcome.public_operations = 0;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cmrp_script_history_and_references_are_bounded_and_do_not_create_authority() {
        let mut history = History::default();
        for i in 0..HISTORY_CAP + 7 {
            history.push(i, Some(json!({"value":i})), 16);
        }
        assert_eq!(history.values.len(), HISTORY_CAP);
        assert_eq!(history.dropped, 7);
        assert!(history
            .get(&ResultPointer {
                step: 0,
                pointer: "/value".into()
            })
            .is_err());
        let step: TrialStep = serde_json::from_value(json!({"kind":"command","command":{"type":"select_map_node","node_id":null},"bindings":[{"from":{"step":264,"pointer":"/value"},"to":"/command/node_id","value_type":"string"}]})).unwrap();
        assert!(validate(&step, 264).is_err());
        assert!(validate(&step, 265).is_ok());
        assert!(resolve(&step, &history).is_err());
        let node = uuid::Uuid::nil().to_string();
        history.push(264, Some(json!({"value":node})), 64);
        let (_, context, resolved) = resolve(&step, &history).unwrap();
        assert_eq!(resolved["command"]["node_id"], node);
        assert!(context.expected_battle_authority.is_none());
        history.push(999, Some(Value::Null), HISTORY_BYTE_CAP + 1);
        assert!(history.bytes <= HISTORY_BYTE_CAP);
        let bad: TrialStep = serde_json::from_value(json!({"kind":"command","command":{"type":"select_map_node","node_id":null},"bindings":[{"from":{"step":264,"pointer":"/value"},"to":"/runtime_safety/max_retained_bytes","value_type":"string"}]})).unwrap();
        assert!(validate(&bad, 265).is_err());
    }
    #[test]
    fn cmrp_script_missing_history_cannot_satisfy_a_missing_pointer_assertion() {
        let mut history = History::default();
        let from = ResultPointer {
            step: 0,
            pointer: "/absent".into(),
        };
        let absent = Assertion::Exists { expected: false };
        assert!(assertion(&from, &absent, &history).is_err());
        history.push(0, Some(json!({"retained":true})), 32);
        assert!(assertion(&from, &absent, &history)
            .unwrap()
            .error_code
            .is_none());
        for i in 1..=HISTORY_CAP {
            history.push(i, Some(Value::Null), 4);
        }
        assert!(assertion(&from, &absent, &history).is_err());
    }
}
