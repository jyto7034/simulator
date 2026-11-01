//! Finite scenario input. Gameplay commands remain owned by ApplicationCommand.

use super::{AuthoringError, ExecutionPhase, RequestFailure};
use crate::application::ApplicationSnapshotKind;
use serde::{
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;
use uuid::Uuid;

/// Trusted host ceilings; scenario documents cannot change them.
#[derive(Debug, Clone, Serialize)]
pub struct ScenarioLimits {
    pub max_input_bytes: usize,
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_steps: usize,
    pub max_bindings_per_step: usize,
    pub max_pointer_bytes: usize,
    pub max_pointer_segments: usize,
    pub max_template_bytes: usize,
    pub max_resolved_bytes: usize,
    pub max_copy_bytes_per_step: usize,
    pub max_total_copy_bytes: usize,
    pub max_clock_delta_ms: u64,
    pub max_total_clock_ms: u64,
}
impl Default for ScenarioLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 8 * 1024 * 1024,
            max_depth: 64,
            max_nodes: 262_144,
            max_steps: 256,
            max_bindings_per_step: 64,
            max_pointer_bytes: 1024,
            max_pointer_segments: 32,
            max_template_bytes: 256 * 1024,
            max_resolved_bytes: 1024 * 1024,
            max_copy_bytes_per_step: 1024 * 1024,
            max_total_copy_bytes: 8 * 1024 * 1024,
            max_clock_delta_ms: 60_000,
            max_total_clock_ms: 3_600_000,
        }
    }
}
impl ScenarioLimits {
    fn validate(&self) -> Result<(), AuthoringError> {
        let hard = Self::default();
        for (value, ceiling) in [
            (self.max_input_bytes as u64, hard.max_input_bytes as u64),
            (self.max_depth as u64, hard.max_depth as u64),
            (self.max_nodes as u64, hard.max_nodes as u64),
            (self.max_steps as u64, hard.max_steps as u64),
            (
                self.max_bindings_per_step as u64,
                hard.max_bindings_per_step as u64,
            ),
            (self.max_pointer_bytes as u64, hard.max_pointer_bytes as u64),
            (
                self.max_pointer_segments as u64,
                hard.max_pointer_segments as u64,
            ),
            (
                self.max_template_bytes as u64,
                hard.max_template_bytes as u64,
            ),
            (
                self.max_resolved_bytes as u64,
                hard.max_resolved_bytes as u64,
            ),
            (
                self.max_copy_bytes_per_step as u64,
                hard.max_copy_bytes_per_step as u64,
            ),
            (
                self.max_total_copy_bytes as u64,
                hard.max_total_copy_bytes as u64,
            ),
            (self.max_clock_delta_ms, hard.max_clock_delta_ms),
            (self.max_total_clock_ms, hard.max_total_clock_ms),
        ] {
            if value == 0 || value > ceiling {
                return Err(error(
                    "authoring_scenario_limits_invalid",
                    "Trusted scenario ceilings must be positive and no larger than their defaults.",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialInitial {
    pub player_id: Uuid,
    pub run_seed: u64,
    pub profile: TrialProfile,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialProfile {
    DefaultMemory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultPointer {
    pub step: usize,
    pub pointer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonValueType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub from: ResultPointer,
    pub to: String,
    pub value_type: JsonValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum TrialClock {
    Elapsed { delta_ms: u64, after_seq: u64 },
    HostTick { delta_ms: u64, after_seq: u64 },
    NextEventBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "test", rename_all = "snake_case", deny_unknown_fields)]
pub enum Assertion {
    Equals { value: Value },
    Exists { expected: bool },
    Type { value_type: JsonValueType },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TrialStep {
    Command {
        command: Value,
        #[serde(default = "empty_object")]
        context: Value,
        #[serde(default)]
        bindings: Vec<Binding>,
    },
    Snapshot {
        snapshot: ApplicationSnapshotKind,
    },
    Clock {
        clock: TrialClock,
    },
    Assert {
        from: ResultPointer,
        assertion: Assertion,
    },
}
fn empty_object() -> Value {
    Value::Object(Default::default())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    initial: TrialInitial,
    preparation_steps: Vec<TrialStep>,
    steps: Vec<TrialStep>,
}

/// A checked finite request. Parsing never creates or advances an application.
#[derive(Debug)]
pub struct TrialScenario {
    document: Document,
    input_bytes: usize,
    input_sha256: String,
    limits: ScenarioLimits,
}
impl TrialScenario {
    pub fn from_json_slice(bytes: &[u8], limits: &ScenarioLimits) -> Result<Self, AuthoringError> {
        limits.validate()?;
        if bytes.len() > limits.max_input_bytes {
            return Err(error(
                "authoring_scenario_input_limit",
                "Scenario input exceeds the trusted byte ceiling.",
            ));
        }
        let mut budget = ParseBudget {
            limits,
            nodes: 0,
            fault: None,
        };
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        let value = Seed {
            budget: &mut budget,
            depth: 0,
        }
        .deserialize(&mut deserializer)
        .and_then(|value| {
            deserializer.end()?;
            Ok(value)
        })
        .map_err(|_| {
            error(
                budget.fault.unwrap_or("authoring_scenario_json_invalid"),
                "Scenario JSON is invalid or exceeds its structural ceiling.",
            )
        })?;
        let document: Document = serde_json::from_value(value).map_err(|_| {
            error(
                "authoring_scenario_schema_invalid",
                "Scenario input does not match the finite request schema.",
            )
        })?;
        validate_document(&document, limits)?;
        Ok(Self {
            document,
            input_bytes: bytes.len(),
            input_sha256: format!("{:x}", Sha256::digest(bytes)),
            limits: limits.clone(),
        })
    }
    pub fn initial(&self) -> &TrialInitial {
        &self.document.initial
    }
    pub fn preparation_steps(&self) -> &[TrialStep] {
        &self.document.preparation_steps
    }
    pub fn steps(&self) -> &[TrialStep] {
        &self.document.steps
    }
    pub fn input_bytes(&self) -> usize {
        self.input_bytes
    }
    pub fn input_sha256(&self) -> &str {
        &self.input_sha256
    }
    pub fn limits(&self) -> &ScenarioLimits {
        &self.limits
    }
    pub(super) fn step(&self, index: usize) -> Option<&TrialStep> {
        if index < self.document.preparation_steps.len() {
            self.document.preparation_steps.get(index)
        } else {
            self.document
                .steps
                .get(index - self.document.preparation_steps.len())
        }
    }
}

struct ParseBudget<'a> {
    limits: &'a ScenarioLimits,
    nodes: usize,
    fault: Option<&'static str>,
}
struct Seed<'a, 'b> {
    budget: &'a mut ParseBudget<'b>,
    depth: usize,
}
impl<'de> DeserializeSeed<'de> for Seed<'_, '_> {
    type Value = Value;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        let fault = if self.depth > self.budget.limits.max_depth {
            Some("authoring_scenario_depth_limit")
        } else if self.budget.nodes >= self.budget.limits.max_nodes {
            Some("authoring_scenario_node_limit")
        } else {
            None
        };
        if let Some(code) = fault {
            self.budget.fault = Some(code);
            return Err(D::Error::custom(code));
        }
        self.budget.nodes += 1;
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed<'_, '_> {
    type Value = Value;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("bounded JSON")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("invalid number"))
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(v.to_owned().into())
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = access.next_element_seed(Seed {
            budget: self.budget,
            depth: self.depth + 1,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Value, A::Error> {
        let mut values = serde_json::Map::new();
        while let Some(key) = access.next_key::<String>()? {
            if values.contains_key(&key) {
                self.budget.fault = Some("authoring_scenario_duplicate_key");
                return Err(A::Error::custom("duplicate object key"));
            }
            let value = access.next_value_seed(Seed {
                budget: self.budget,
                depth: self.depth + 1,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

pub(super) fn pointer(
    value: &str,
    limits: &ScenarioLimits,
    root_allowed: bool,
) -> Result<Vec<String>, AuthoringError> {
    if value.len() > limits.max_pointer_bytes {
        return Err(error(
            "authoring_pointer_limit",
            "JSON pointer exceeds its byte ceiling.",
        ));
    }
    if value.is_empty() && root_allowed {
        return Ok(Vec::new());
    }
    if !value.starts_with('/') {
        return Err(error(
            "authoring_pointer_invalid",
            "JSON pointer must start with a slash.",
        ));
    }
    let mut tokens = Vec::new();
    for raw in value[1..].split('/') {
        if tokens.len() == limits.max_pointer_segments {
            return Err(error(
                "authoring_pointer_limit",
                "JSON pointer has too many segments.",
            ));
        }
        let mut decoded = String::new();
        let mut chars = raw.chars();
        while let Some(ch) = chars.next() {
            if ch != '~' {
                decoded.push(ch);
                continue;
            }
            decoded.push(match chars.next() {
                Some('0') => '~',
                Some('1') => '/',
                _ => {
                    return Err(error(
                        "authoring_pointer_invalid",
                        "JSON pointer escape must be ~0 or ~1.",
                    ))
                }
            });
        }
        tokens.push(decoded);
    }
    Ok(tokens)
}

pub(super) fn array_index(token: &str) -> Option<usize> {
    if token.is_empty()
        || (token.len() > 1 && token.starts_with('0'))
        || !token.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    token.parse().ok()
}

pub(super) fn at_pointer<'a>(mut value: &'a Value, tokens: &[String]) -> Option<&'a Value> {
    for token in tokens {
        value = match value {
            Value::Object(map) => map.get(token)?,
            Value::Array(array) => array.get(array_index(token)?)?,
            _ => return None,
        };
    }
    Some(value)
}

fn validate_source(
    from: &ResultPointer,
    index: usize,
    all: &[&TrialStep],
    limits: &ScenarioLimits,
) -> Result<(), AuthoringError> {
    if from.step >= index || matches!(all.get(from.step), Some(TrialStep::Assert { .. }) | None) {
        return Err(error(
            "authoring_binding_source_invalid",
            "A source must be an earlier public command, snapshot or clock result.",
        ));
    }
    pointer(&from.pointer, limits, true)?;
    Ok(())
}

fn validate_document(document: &Document, limits: &ScenarioLimits) -> Result<(), AuthoringError> {
    if document
        .preparation_steps
        .len()
        .saturating_add(document.steps.len())
        > limits.max_steps
    {
        return Err(error(
            "authoring_scenario_step_limit",
            "Scenario contains too many steps.",
        ));
    }
    let all: Vec<_> = document
        .preparation_steps
        .iter()
        .chain(&document.steps)
        .collect();
    let mut clock_total = 0_u64;
    for (index, step) in all.iter().enumerate() {
        match step {
            TrialStep::Command {
                command,
                context,
                bindings,
            } => {
                if !command.is_object()
                    || !command.get("type").is_some_and(Value::is_string)
                    || !context.is_object()
                {
                    return Err(error(
                        "authoring_command_template_invalid",
                        "Command needs a string type and command/context must be objects.",
                    ));
                }
                if bindings.len() > limits.max_bindings_per_step {
                    return Err(error(
                        "authoring_binding_limit",
                        "Command contains too many bindings.",
                    ));
                }
                #[derive(Serialize)]
                struct Template<'a> {
                    command: &'a Value,
                    context: &'a Value,
                }
                measure(&Template { command, context }, limits.max_template_bytes)?;
                validate_context(context)?;
                let mut destinations = Vec::new();
                for binding in bindings {
                    validate_source(&binding.from, index, &all, limits)?;
                    let tokens = pointer(&binding.to, limits, false)?;
                    if tokens.len() < 2 || (tokens[0] == "command" && tokens[1] == "type") {
                        return Err(error(
                            "authoring_binding_destination_invalid",
                            "Bindings cannot replace a command/context root or command type.",
                        ));
                    }
                    let root = match tokens[0].as_str() {
                        "command" => command,
                        "context" => context,
                        _ => {
                            return Err(error(
                                "authoring_binding_destination_invalid",
                                "Binding destination must be inside command or context.",
                            ))
                        }
                    };
                    if at_pointer(root, &tokens[1..]).is_none() {
                        return Err(error("authoring_binding_destination_missing", "Binding destination must already exist; append and parent creation are unsupported."));
                    }
                    destinations.push(tokens);
                }
                destinations.sort();
                if destinations
                    .windows(2)
                    .any(|pair| pair[1].starts_with(&pair[0]))
                {
                    return Err(error(
                        "authoring_binding_destination_overlap",
                        "Binding destinations must not duplicate or contain one another.",
                    ));
                }
            }
            TrialStep::Clock { clock } => {
                let delta = match clock {
                    TrialClock::Elapsed { delta_ms, .. }
                    | TrialClock::HostTick { delta_ms, .. } => *delta_ms,
                    TrialClock::NextEventBucket => 0,
                };
                clock_total = clock_total.checked_add(delta).ok_or_else(|| {
                    error("authoring_clock_limit", "Requested clock sum overflows.")
                })?;
                if delta > limits.max_clock_delta_ms || clock_total > limits.max_total_clock_ms {
                    return Err(error(
                        "authoring_clock_limit",
                        "Requested clock exceeds its trusted ceiling.",
                    ));
                }
            }
            TrialStep::Assert { from, assertion } => {
                validate_source(from, index, &all, limits)?;
                measure(assertion, limits.max_template_bytes)?;
            }
            TrialStep::Snapshot { .. } => {}
        }
    }
    Ok(())
}

fn validate_context(context: &Value) -> Result<(), AuthoringError> {
    let Some(fields) = context.as_object() else {
        return Err(error(
            "authoring_context_template_invalid",
            "Execution context must be an object.",
        ));
    };
    for (key, value) in fields {
        let valid = match key.as_str() {
            "source_command_id" => value.is_null() || value.is_string(),
            "battle_event_after_seq" => value.is_null() || value.as_u64().is_some(),
            "expected_battle_authority" => {
                value.is_null()
                    || value.as_object().is_some_and(|fields| {
                        fields.iter().all(|(key, value)| match key.as_str() {
                            "battle_uuid" => value.is_null() || value.is_string(),
                            "battle_state_revision" => value.is_null() || value.as_u64().is_some(),
                            _ => false,
                        })
                    })
            }
            _ => false,
        };
        if !valid {
            return Err(error(
                "authoring_context_template_invalid",
                "Execution context contains an unknown field or incompatible template value.",
            ));
        }
    }
    Ok(())
}

fn measure(value: &impl Serialize, limit: usize) -> Result<(), AuthoringError> {
    struct Counter {
        count: usize,
        limit: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.count = self
                .count
                .checked_add(bytes.len())
                .filter(|count| *count <= self.limit)
                .ok_or_else(|| std::io::Error::other("template limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { count: 0, limit }, value).map_err(|_| {
        error(
            "authoring_template_limit",
            "Template exceeds its encoded byte ceiling.",
        )
    })
}

fn error(code: &'static str, reason: &str) -> AuthoringError {
    AuthoringError::Request {
        diagnostic: RequestFailure {
            code,
            phase: ExecutionPhase::TrialPreparation,
            field_path: "scenario",
            reason: reason.to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn minimal() -> Value {
        json!({"initial": {"player_id": "00000000-0000-0000-0000-000000000001", "run_seed": 17, "profile": "default_memory"}, "preparation_steps": [], "steps": []})
    }
    fn parse(value: &Value, limits: &ScenarioLimits) -> Result<TrialScenario, AuthoringError> {
        TrialScenario::from_json_slice(&serde_json::to_vec(value).unwrap(), limits)
    }
    fn code(result: Result<TrialScenario, AuthoringError>) -> &'static str {
        match result {
            Err(AuthoringError::Request { diagnostic }) => diagnostic.code,
            other => panic!("expected request rejection, got {other:?}"),
        }
    }
    fn binding_case() -> Value {
        let mut value = minimal();
        value["steps"] = json!([
            {"kind": "snapshot", "snapshot": "run"},
            {"kind": "command", "command": {"type": "request_battle_state", "payload": {"a/b": null, "node": {"leaf": null}, "array": [null, null]}},
             "bindings": [{"from": {"step": 0, "pointer": "/read_model/a~1b"}, "to": "/command/payload/a~1b", "value_type": "string"}]}
        ]);
        value
    }

    #[test]
    fn finite_input_preserves_explicit_initial_and_checks_exact_byte_node_depth_caps() {
        let value = minimal();
        let bytes = serde_json::to_vec(&value).unwrap();
        let limits = ScenarioLimits {
            max_input_bytes: bytes.len(),
            max_nodes: 7,
            max_depth: 2,
            ..Default::default()
        };
        let scenario = TrialScenario::from_json_slice(&bytes, &limits).unwrap();
        assert_eq!(scenario.initial().run_seed, 17);
        assert_eq!(scenario.initial().player_id, Uuid::from_u128(1));
        assert!(scenario.steps().is_empty());
        assert!(scenario.preparation_steps().is_empty());
        assert_eq!(scenario.input_bytes(), bytes.len());
        assert_eq!(
            scenario.input_sha256(),
            format!("{:x}", Sha256::digest(&bytes))
        );
        assert_eq!(
            code(TrialScenario::from_json_slice(
                &bytes,
                &ScenarioLimits {
                    max_input_bytes: bytes.len() - 1,
                    ..limits.clone()
                }
            )),
            "authoring_scenario_input_limit"
        );
        assert_eq!(
            code(parse(
                &value,
                &ScenarioLimits {
                    max_nodes: 6,
                    ..limits.clone()
                }
            )),
            "authoring_scenario_node_limit"
        );
        assert_eq!(
            code(parse(
                &value,
                &ScenarioLimits {
                    max_depth: 1,
                    ..limits
                }
            )),
            "authoring_scenario_depth_limit"
        );
        assert_eq!(
            code(parse(
                &value,
                &ScenarioLimits {
                    max_steps: 0,
                    ..Default::default()
                }
            )),
            "authoring_scenario_limits_invalid"
        );
    }

    #[test]
    fn duplicate_keys_unknown_envelopes_and_unsupported_profiles_are_rejected() {
        let limits = ScenarioLimits::default();
        assert_eq!(
            code(TrialScenario::from_json_slice(
                br#"{"initial":{"run_seed":1,"run_seed":2}}"#,
                &limits
            )),
            "authoring_scenario_duplicate_key"
        );
        assert_eq!(
            code(TrialScenario::from_json_slice(
                br#"{"initial":{},"initial":{}}"#,
                &limits
            )),
            "authoring_scenario_duplicate_key"
        );
        for (key, changed) in [
            ("extra", json!(true)),
            (
                "initial",
                json!({"player_id":"00000000-0000-0000-0000-000000000001","run_seed":17,"profile":"personal_file"}),
            ),
            ("steps", json!([{"kind":"repeat","times":3}])),
        ] {
            let mut value = minimal();
            value[key] = changed;
            assert_eq!(
                code(parse(&value, &limits)),
                "authoring_scenario_schema_invalid"
            );
        }
        let mut value = binding_case();
        value["steps"][1]["context"] = json!({"private_authority": true});
        assert_eq!(
            code(parse(&value, &limits)),
            "authoring_context_template_invalid"
        );
    }

    #[test]
    fn bindings_require_prior_public_sources_and_existing_disjoint_destinations() {
        let limits = ScenarioLimits::default();
        let original = binding_case();
        parse(&original, &limits).unwrap();
        for (to, expected) in [
            ("/command/type", "authoring_binding_destination_invalid"),
            ("/command", "authoring_binding_destination_invalid"),
            ("/initial/run_seed", "authoring_binding_destination_invalid"),
            (
                "/command/payload/missing/leaf",
                "authoring_binding_destination_missing",
            ),
            (
                "/command/payload/array/-",
                "authoring_binding_destination_missing",
            ),
            (
                "/command/payload/array/01",
                "authoring_binding_destination_missing",
            ),
            ("/command/payload/a~2b", "authoring_pointer_invalid"),
        ] {
            let mut value = original.clone();
            value["steps"][1]["bindings"][0]["to"] = to.into();
            assert_eq!(code(parse(&value, &limits)), expected, "{to}");
        }
        let mut forward = original.clone();
        forward["steps"][1]["bindings"][0]["from"]["step"] = 1.into();
        assert_eq!(
            code(parse(&forward, &limits)),
            "authoring_binding_source_invalid"
        );
        let mut assertion_source = original.clone();
        assertion_source["steps"][1] = json!({"kind":"assert", "from":{"step":0,"pointer":""}, "assertion":{"test":"exists","expected":true}});
        let mut command = original["steps"][1].clone();
        command["bindings"][0]["from"]["step"] = 1.into();
        assertion_source["steps"]
            .as_array_mut()
            .unwrap()
            .push(command);
        assert_eq!(
            code(parse(&assertion_source, &limits)),
            "authoring_binding_source_invalid"
        );
        for destinations in [
            ["/command/payload/node", "/command/payload/node/leaf"],
            ["/command/payload/a~1b", "/command/payload/a~1b"],
        ] {
            let mut value = original.clone();
            value["steps"][1]["bindings"] = destinations
                .into_iter()
                .map(|to| json!({"from":{"step":0,"pointer":""},"to":to,"value_type":"object"}))
                .collect::<Vec<_>>()
                .into();
            assert_eq!(
                code(parse(&value, &limits)),
                "authoring_binding_destination_overlap"
            );
        }
    }

    #[test]
    fn step_binding_pointer_template_and_clock_ceilings_are_inclusive() {
        let value = binding_case();
        let mut limits = ScenarioLimits {
            max_steps: 2,
            max_bindings_per_step: 1,
            max_pointer_bytes: 21,
            max_pointer_segments: 3,
            ..Default::default()
        };
        parse(&value, &limits).unwrap();
        limits.max_steps = 1;
        assert_eq!(
            code(parse(&value, &limits)),
            "authoring_scenario_step_limit"
        );
        limits.max_steps = 2;
        limits.max_pointer_bytes = 19;
        assert_eq!(code(parse(&value, &limits)), "authoring_pointer_limit");
        limits.max_pointer_bytes = 21;
        limits.max_pointer_segments = 2;
        assert_eq!(code(parse(&value, &limits)), "authoring_pointer_limit");
        limits.max_pointer_segments = 3;
        limits.max_template_bytes = 1;
        assert_eq!(code(parse(&value, &limits)), "authoring_template_limit");
        let mut clocks = minimal();
        clocks["steps"] = json!([{"kind":"clock","clock":{"mode":"elapsed","delta_ms":60000,"after_seq":0}}, {"kind":"clock","clock":{"mode":"host_tick","delta_ms":60000,"after_seq":0}}]);
        let exact = ScenarioLimits {
            max_total_clock_ms: 120_000,
            ..Default::default()
        };
        parse(&clocks, &exact).unwrap();
        assert_eq!(
            code(parse(
                &clocks,
                &ScenarioLimits {
                    max_total_clock_ms: 119_999,
                    ..exact.clone()
                }
            )),
            "authoring_clock_limit"
        );
        clocks["steps"][0]["clock"]["delta_ms"] = u64::MAX.into();
        assert_eq!(code(parse(&clocks, &exact)), "authoring_clock_limit");
    }
}
