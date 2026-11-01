//! Private preparation for a trial command. The session supplies only committed
//! public result slots; neither receipts nor optional observations are inputs.

use super::{
    control, scenario, AuthoringError, ExecutionPhase, RequestFailure, ScenarioLimits,
    TrialScenario, TrialStep,
};
use crate::application::{
    ApplicationBattleAuthority, ApplicationCommand, ApplicationDocument,
    ApplicationExecutionContext,
};
use serde::{
    ser::{Error as _, SerializeMap, SerializeSeq},
    Deserialize, Serialize, Serializer,
};
use serde_json::Value;
use std::{cell::RefCell, collections::BTreeMap};
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct ResolvedCommand {
    pub command: ApplicationCommand,
    pub context: ApplicationExecutionContext,
    pub resolved_context: Value,
    /// Candidate total; the resolver never changes the committed quota.
    pub total_copy_bytes: usize,
}

#[derive(Default)]
struct Patch<'a> {
    replacement: Option<&'a Value>,
    children: BTreeMap<String, Patch<'a>>,
}
impl<'a> Patch<'a> {
    fn insert(&mut self, tokens: Vec<String>, value: &'a Value) {
        let mut node = self;
        for token in tokens {
            node = node.children.entry(token).or_default();
        }
        node.replacement = Some(value);
    }
}

fn error(phase: ExecutionPhase, code: &'static str, reason: &'static str) -> AuthoringError {
    AuthoringError::Request {
        diagnostic: RequestFailure {
            code,
            phase,
            field_path: "scenario",
            reason: reason.into(),
        },
    }
}

fn charge(units: usize, phase: ExecutionPhase, site: &'static str) -> Result<(), AuthoringError> {
    control::charge(units as u64, site).map_err(|_| {
        error(
            phase,
            "authoring_binding_interrupted",
            "Execution control interrupted command preparation.",
        )
    })
}

fn matches_type(value: &Value, kind: super::JsonValueType) -> bool {
    use super::JsonValueType as K;
    matches!(
        (value, kind),
        (Value::Null, K::Null)
            | (Value::Bool(_), K::Bool)
            | (Value::Number(_), K::Number)
            | (Value::String(_), K::String)
            | (Value::Array(_), K::Array)
            | (Value::Object(_), K::Object)
    )
}

fn source_at<'a>(
    from: &super::ResultPointer,
    public_results: &[Option<&'a Value>],
    limits: &ScenarioLimits,
    phase: ExecutionPhase,
) -> Result<Option<&'a Value>, AuthoringError> {
    charge(from.pointer.len(), phase, "binding_pointer_decode")?;
    let mut value = public_results
        .get(from.step)
        .and_then(|slot| *slot)
        .ok_or_else(|| {
            error(
                phase,
                "authoring_binding_source_unavailable",
                "The source step has no accepted public result.",
            )
        })?;
    let tokens = scenario::pointer(&from.pointer, limits, true)?;
    for token in &tokens {
        let width = match value {
            Value::Object(map) => map.len(),
            _ => 1,
        };
        charge(
            width
                .saturating_add(1)
                .saturating_mul(token.len().saturating_add(1)),
            phase,
            "binding_source_lookup",
        )?;
        if value.is_array() && scenario::array_index(token).is_none() {
            return Err(error(
                phase,
                "authoring_pointer_invalid",
                "An array pointer token must be a canonical in-range integer representation.",
            ));
        }
        let Some(next) = scenario::at_pointer(value, std::slice::from_ref(token)) else {
            return Ok(None);
        };
        value = next;
    }
    Ok(Some(value))
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(super) struct AssertionObservation {
    pub exists: bool,
    pub value_type: Option<super::JsonValueType>,
}

/// An assertion observes a prior public value; it never creates a new binding
/// source or copies that value into the result history.
pub(super) fn resolve_assertion(
    scenario: &TrialScenario,
    index: usize,
    public_results: &[Option<&Value>],
) -> Result<AssertionObservation, AuthoringError> {
    use super::{Assertion, JsonValueType as K};
    let phase = if index < scenario.preparation_steps().len() {
        ExecutionPhase::TrialPreparation
    } else {
        ExecutionPhase::TrialExecution
    };
    if !control::is_active() {
        return Err(error(
            phase,
            "authoring_scope_required",
            "Assertion preparation requires an active authoring attempt.",
        ));
    }
    let _phase = control::phase(phase);
    if public_results.len() != index {
        return Err(error(
            phase,
            "authoring_trial_basis_invalid",
            "Accepted result count does not match this step.",
        ));
    }
    let Some(TrialStep::Assert { from, assertion }) = scenario.step(index) else {
        return Err(error(
            phase,
            "authoring_step_kind_invalid",
            "The selected step is not an assertion.",
        ));
    };
    let actual = source_at(from, public_results, scenario.limits(), phase)?;
    let value_type = actual.map(|value| match value {
        Value::Null => K::Null,
        Value::Bool(_) => K::Bool,
        Value::Number(_) => K::Number,
        Value::String(_) => K::String,
        Value::Array(_) => K::Array,
        Value::Object(_) => K::Object,
    });
    let passed = match assertion {
        Assertion::Exists { expected } => actual.is_some() == *expected,
        Assertion::Type {
            value_type: expected,
        } => value_type == Some(*expected),
        Assertion::Equals { value } => match actual {
            Some(actual) => {
                let limits = scenario.limits();
                let left = measure(actual, None, limits, limits.max_resolved_bytes, phase)?;
                let right = measure(value, None, limits, limits.max_resolved_bytes, phase)?;
                charge(
                    left.bytes
                        .saturating_add(left.nodes)
                        .saturating_add(right.bytes)
                        .saturating_add(right.nodes),
                    phase,
                    "assertion_value_compare",
                )?;
                actual == value
            }
            None => false,
        },
    };
    if !passed {
        return Err(error(
            phase,
            "authoring_assertion_failed",
            "The accepted public result does not satisfy the assertion.",
        ));
    }
    Ok(AssertionObservation {
        exists: actual.is_some(),
        value_type,
    })
}

/// This is deliberately not a public raw-step executor. `public_results` is a
/// borrowed view of the accepted aggregate, with None for steps without a public
/// result. The checked scenario owns chronology and destination validation.
pub(super) fn resolve_command(
    scenario: &TrialScenario,
    index: usize,
    public_results: &[Option<&Value>],
    committed_copy_bytes: usize,
) -> Result<ResolvedCommand, AuthoringError> {
    let phase = if index < scenario.preparation_steps().len() {
        ExecutionPhase::TrialPreparation
    } else {
        ExecutionPhase::TrialExecution
    };
    let limits = scenario.limits();
    if !control::is_active() {
        return Err(error(
            phase,
            "authoring_scope_required",
            "Command preparation requires an active authoring attempt.",
        ));
    }
    let _phase = control::phase(phase);
    if public_results.len() != index || committed_copy_bytes > limits.max_total_copy_bytes {
        return Err(error(
            phase,
            "authoring_trial_basis_invalid",
            "Accepted result count or copy quota does not match this step.",
        ));
    }
    let Some(TrialStep::Command {
        command,
        context,
        bindings,
    }) = scenario.step(index)
    else {
        return Err(error(
            phase,
            "authoring_step_kind_invalid",
            "The selected step is not a command.",
        ));
    };
    let mut patch = Patch::default();
    let mut copied = 0_usize;
    for binding in bindings {
        charge(binding.to.len(), phase, "binding_pointer_decode")?;
        let value = source_at(&binding.from, public_results, limits, phase)?.ok_or_else(|| {
            error(
                phase,
                "authoring_binding_source_missing",
                "The source JSON pointer does not exist.",
            )
        })?;
        if !matches_type(value, binding.value_type) {
            return Err(error(
                phase,
                "authoring_binding_type_mismatch",
                "The public source does not have the declared JSON type.",
            ));
        }
        let stats = measure(value, None, limits, limits.max_copy_bytes_per_step, phase)?;
        copied = copied.checked_add(stats.bytes).ok_or_else(|| {
            error(
                phase,
                "authoring_binding_copy_limit",
                "Copied source byte count overflows.",
            )
        })?;
        if copied > limits.max_copy_bytes_per_step
            || copied > limits.max_total_copy_bytes - committed_copy_bytes
        {
            return Err(error(
                phase,
                "authoring_binding_copy_limit",
                "Copied sources exceed the step or session byte ceiling.",
            ));
        }
        patch.insert(scenario::pointer(&binding.to, limits, false)?, value);
    }

    let budget = RefCell::new(ShapeBudget::new(limits, limits.max_resolved_bytes, phase));
    let request = RequestView {
        command,
        context,
        patch: &patch,
        budget: &budget,
    };
    let stats = measure_serializable(&request, &budget)?;
    // One materialized request and the public decoder's owned document/domain
    // copies. This is source-sized logical work, not allocator accounting.
    charge(
        stats.bytes.saturating_add(stats.nodes).saturating_mul(3),
        phase,
        "binding_materialize_and_decode",
    )?;
    let resolved_command = materialize(command, patch.children.get("command"));
    let resolved_context = materialize(context, patch.children.get("context"));
    let context = decode_context(&resolved_context, phase)?;
    let command = ApplicationCommand::from_document(ApplicationDocument::new(resolved_command))
        .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?;
    Ok(ResolvedCommand {
        command,
        context,
        resolved_context,
        total_copy_bytes: committed_copy_bytes + copied,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    #[serde(default)]
    source_command_id: Option<String>,
    #[serde(default)]
    battle_event_after_seq: u64,
    #[serde(default)]
    expected_battle_authority: Option<Authority>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authority {
    battle_uuid: Uuid,
    battle_state_revision: u64,
}
fn decode_context(
    value: &Value,
    phase: ExecutionPhase,
) -> Result<ApplicationExecutionContext, AuthoringError> {
    let parsed = Context::deserialize(value).map_err(|_| {
        error(
            phase,
            "authoring_context_invalid",
            "Resolved execution context does not match the public fields and types.",
        )
    })?;
    Ok(ApplicationExecutionContext {
        source_command_id: parsed.source_command_id,
        battle_event_after_seq: parsed.battle_event_after_seq,
        expected_battle_authority: parsed.expected_battle_authority.map(|value| {
            ApplicationBattleAuthority {
                battle_uuid: value.battle_uuid,
                battle_state_revision: value.battle_state_revision,
            }
        }),
    })
}

#[derive(Clone, Copy)]
struct Stats {
    bytes: usize,
    nodes: usize,
}
struct ShapeBudget<'a> {
    limits: &'a ScenarioLimits,
    max_bytes: usize,
    phase: ExecutionPhase,
    bytes: usize,
    nodes: usize,
    failure: Option<AuthoringError>,
}
impl<'a> ShapeBudget<'a> {
    fn new(limits: &'a ScenarioLimits, max_bytes: usize, phase: ExecutionPhase) -> Self {
        Self {
            limits,
            max_bytes,
            phase,
            bytes: 0,
            nodes: 0,
            failure: None,
        }
    }
    fn reject(&mut self, code: &'static str, reason: &'static str) -> Result<(), ()> {
        if self.failure.is_none() {
            self.failure = Some(error(self.phase, code, reason));
        }
        Err(())
    }
    fn node(&mut self, depth: usize) -> Result<(), ()> {
        if depth > self.limits.max_depth {
            return self.reject(
                "authoring_resolved_depth_limit",
                "Resolved JSON exceeds the depth ceiling.",
            );
        }
        if self.nodes == self.limits.max_nodes {
            return self.reject(
                "authoring_resolved_node_limit",
                "Resolved JSON exceeds the node ceiling.",
            );
        }
        control::charge(1, "binding_shape_node").map_err(|_| ())?;
        self.nodes += 1;
        Ok(())
    }
}

/// Both traversal and byte admission use a borrowed projection. A replacement's
/// depth is checked at its actual destination before descending into it.
struct Project<'a, 'v> {
    value: &'v Value,
    patch: Option<&'a Patch<'v>>,
    depth: usize,
    budget: &'a RefCell<ShapeBudget<'a>>,
}
impl Serialize for Project<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.budget
            .borrow_mut()
            .node(self.depth)
            .map_err(|()| S::Error::custom("bounded binding shape"))?;
        let (value, patch) = match self.patch.and_then(|patch| patch.replacement) {
            Some(value) => (value, None),
            None => (self.value, self.patch),
        };
        match value {
            Value::Object(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (key, value) in values {
                    if let Some(patch) = patch.filter(|p| !p.children.is_empty()) {
                        // Covers this lookup and the later materialization lookup.
                        control::charge(
                            (key.len()
                                .saturating_add(1)
                                .saturating_mul(patch.children.len().saturating_add(1))
                                .saturating_mul(2)) as u64,
                            "binding_patch_lookup",
                        )
                        .map_err(|_| S::Error::custom("binding control interrupted"))?;
                    }
                    let child = patch.and_then(|p| p.children.get(key));
                    map.serialize_entry(
                        key,
                        &Project {
                            value,
                            patch: child,
                            depth: self.depth + 1,
                            budget: self.budget,
                        },
                    )?;
                }
                map.end()
            }
            Value::Array(values) => {
                let mut array = serializer.serialize_seq(Some(values.len()))?;
                for (index, value) in values.iter().enumerate() {
                    if let Some(patch) = patch.filter(|p| !p.children.is_empty()) {
                        control::charge(
                            (21_usize
                                .saturating_mul(patch.children.len().saturating_add(1))
                                .saturating_mul(2)) as u64,
                            "binding_patch_lookup",
                        )
                        .map_err(|_| S::Error::custom("binding control interrupted"))?;
                    }
                    let child = patch
                        .filter(|p| !p.children.is_empty())
                        .and_then(|p| p.children.get(&index.to_string()));
                    array.serialize_element(&Project {
                        value,
                        patch: child,
                        depth: self.depth + 1,
                        budget: self.budget,
                    })?;
                }
                array.end()
            }
            scalar => scalar.serialize(serializer),
        }
    }
}

struct RequestView<'a, 'v> {
    command: &'v Value,
    context: &'v Value,
    patch: &'a Patch<'v>,
    budget: &'a RefCell<ShapeBudget<'a>>,
}
impl Serialize for RequestView<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.budget
            .borrow_mut()
            .node(0)
            .map_err(|()| S::Error::custom("bounded binding root"))?;
        let mut map = serializer.serialize_map(Some(2))?;
        for (key, value) in [("command", self.command), ("context", self.context)] {
            map.serialize_entry(
                key,
                &Project {
                    value,
                    patch: self.patch.children.get(key),
                    depth: 1,
                    budget: self.budget,
                },
            )?;
        }
        map.end()
    }
}

fn measure(
    value: &Value,
    patch: Option<&Patch<'_>>,
    limits: &ScenarioLimits,
    max_bytes: usize,
    phase: ExecutionPhase,
) -> Result<Stats, AuthoringError> {
    let budget = RefCell::new(ShapeBudget::new(limits, max_bytes, phase));
    measure_serializable(
        &Project {
            value,
            patch,
            depth: 0,
            budget: &budget,
        },
        &budget,
    )
}
fn measure_serializable(
    value: &impl Serialize,
    budget: &RefCell<ShapeBudget<'_>>,
) -> Result<Stats, AuthoringError> {
    struct Counter<'a, 'b>(&'a RefCell<ShapeBudget<'b>>);
    impl std::io::Write for Counter<'_, '_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let mut budget = self.0.borrow_mut();
            if bytes.len() > budget.max_bytes - budget.bytes {
                let _ = budget.reject(
                    "authoring_resolved_byte_limit",
                    "Resolved JSON exceeds the encoded byte ceiling.",
                );
                return Err(std::io::Error::other("bounded binding bytes"));
            }
            control::charge(bytes.len() as u64, "binding_shape_bytes")
                .map_err(|_| std::io::Error::other("binding control interrupted"))?;
            budget.bytes += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let result = serde_json::to_writer(Counter(budget), value);
    let mut budget = budget.borrow_mut();
    if result.is_err() {
        return Err(budget.failure.take().unwrap_or_else(|| {
            error(
                budget.phase,
                "authoring_binding_interrupted",
                "Execution control interrupted command preparation.",
            )
        }));
    }
    Ok(Stats {
        bytes: budget.bytes,
        nodes: budget.nodes,
    })
}

fn materialize(value: &Value, patch: Option<&Patch<'_>>) -> Value {
    if let Some(value) = patch.and_then(|patch| patch.replacement) {
        return value.clone();
    }
    match (value, patch.filter(|p| !p.children.is_empty())) {
        (Value::Object(map), Some(patch)) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), materialize(value, patch.children.get(key))))
                .collect(),
        ),
        (Value::Array(array), Some(patch)) => Value::Array(
            array
                .iter()
                .enumerate()
                .map(|(index, value)| materialize(value, patch.children.get(&index.to_string())))
                .collect(),
        ),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::ApplicationCommandKind,
        authoring::{CancellationToken, ExecutionLimits, ExecutionUsage},
    };
    use serde_json::json;

    fn input(command: Value, context: Value, bindings: Value) -> Value {
        json!({
            "initial": {"player_id": "00000000-0000-0000-0000-000000000001", "run_seed": 17, "profile": "default_memory"},
            "preparation_steps": [{"kind":"snapshot", "snapshot":"run"}],
            "steps": [{"kind":"command", "command":command, "context":context, "bindings":bindings}],
        })
    }
    fn parse(value: &Value, limits: &ScenarioLimits) -> TrialScenario {
        TrialScenario::from_json_slice(&serde_json::to_vec(value).unwrap(), limits).unwrap()
    }
    fn run(
        scenario: &TrialScenario,
        sources: &[Option<&Value>],
        copied: usize,
        limits: &ExecutionLimits,
        token: &CancellationToken,
    ) -> Result<(ResolvedCommand, ExecutionUsage), AuthoringError> {
        let (result, usage) = control::run(limits, token, ExecutionPhase::TrialExecution, || {
            resolve_command(scenario, 1, sources, copied)
        })
        .map_err(|diagnostic| AuthoringError::Execution { diagnostic })?;
        result.map(|result| (result, usage))
    }
    fn resolved(
        scenario: &TrialScenario,
        source: Option<&Value>,
        copied: usize,
    ) -> Result<(ResolvedCommand, ExecutionUsage), AuthoringError> {
        run(
            scenario,
            &[source],
            copied,
            &ExecutionLimits::default(),
            &CancellationToken::default(),
        )
    }
    fn code<T: std::fmt::Debug>(result: Result<T, AuthoringError>) -> String {
        match result.unwrap_err() {
            AuthoringError::Request { diagnostic } => diagnostic.code.into(),
            AuthoringError::Execution { diagnostic } => diagnostic.code.into(),
            AuthoringError::Application { diagnostic, .. } => diagnostic.code().into(),
            other => panic!("unexpected error {other:?}"),
        }
    }
    fn string_binding(pointer: &str, to: &str) -> Value {
        json!({"from":{"step":0,"pointer":pointer}, "to":to, "value_type":"string"})
    }

    #[test]
    fn prior_values_resolve_to_real_command_and_context_without_mutating_the_basis() {
        let source = json!({"a/b":{"~key":["one", "two", "three"]}, "authority":{
            "battle_uuid":"00000000-0000-0000-0000-000000000008", "battle_state_revision":19}, "seq":23});
        let original_source = source.clone();
        let document = input(
            json!({"type":"select_starter_employees", "loan_copy_ids":[], "employee_definition_ids":[null,null,null]}),
            json!({"expected_battle_authority":null, "battle_event_after_seq":null}),
            json!([
                string_binding("/a~1b/~0key/0", "/command/employee_definition_ids/0"),
                string_binding("/a~1b/~0key/1", "/command/employee_definition_ids/1"),
                string_binding("/a~1b/~0key/2", "/command/employee_definition_ids/2"),
                {"from":{"step":0,"pointer":"/authority"},"to":"/context/expected_battle_authority","value_type":"object"},
                {"from":{"step":0,"pointer":"/seq"},"to":"/context/battle_event_after_seq","value_type":"number"},
            ]),
        );
        let scenario = parse(&document, &ScenarioLimits::default());
        let before = format!("{scenario:?}");
        let (result, usage) = resolved(&scenario, Some(&source), 7).unwrap();
        assert_eq!(
            result.command.kind(),
            ApplicationCommandKind::SelectStarterEmployees
        );
        assert_eq!(
            result.command.document().as_value()["employee_definition_ids"],
            json!(["one", "two", "three"])
        );
        assert_eq!(result.context.battle_event_after_seq, 23);
        assert_eq!(
            result
                .context
                .expected_battle_authority
                .unwrap()
                .battle_uuid,
            Uuid::from_u128(8)
        );
        assert_eq!(
            result.resolved_context["expected_battle_authority"],
            source["authority"]
        );
        let source_bytes: usize = [
            json!("one"),
            json!("two"),
            json!("three"),
            source["authority"].clone(),
            json!(23),
        ]
        .iter()
        .map(|v| serde_json::to_vec(v).unwrap().len())
        .sum();
        assert_eq!(result.total_copy_bytes, 7 + source_bytes);
        assert!(usage.work > 0);
        let (_, repeated) = resolved(&scenario, Some(&source), 7).unwrap();
        assert_eq!(usage, repeated);
        assert_eq!(source, original_source);
        assert_eq!(format!("{scenario:?}"), before);
    }

    #[test]
    fn missing_public_result_pointer_and_wrong_source_type_are_errors() {
        let document = input(
            json!({"type":"request_map_data"}),
            json!({"source_command_id":null}),
            json!([string_binding("/01", "/context/source_command_id")]),
        );
        let scenario = parse(&document, &ScenarioLimits::default());
        assert_eq!(
            code(resolved(&scenario, None, 0)),
            "authoring_binding_source_unavailable"
        );
        assert_eq!(
            code(resolved(&scenario, Some(&json!(["a", "b"])), 0)),
            "authoring_pointer_invalid"
        );
        assert_eq!(
            code(resolved(&scenario, Some(&json!({"01":3})), 0)),
            "authoring_binding_type_mismatch"
        );
        assert_eq!(
            resolved(&scenario, Some(&json!({"01":"object key"})), 0)
                .unwrap()
                .0
                .context
                .source_command_id
                .as_deref(),
            Some("object key")
        );
        assert_eq!(
            code(run(
                &scenario,
                &[],
                0,
                &ExecutionLimits::default(),
                &CancellationToken::default()
            )),
            "authoring_trial_basis_invalid"
        );
    }

    #[test]
    fn strict_context_decode_runs_after_whole_authority_binding() {
        let document = input(
            json!({"type":"request_map_data"}),
            json!({"expected_battle_authority":null}),
            json!([{"from":{"step":0,"pointer":""},"to":"/context/expected_battle_authority","value_type":"object"}]),
        );
        let scenario = parse(&document, &ScenarioLimits::default());
        for bad in [
            json!({"battle_uuid":"00000000-0000-0000-0000-000000000008","battle_state_revision":0,"private":true}),
            json!({"battle_uuid":"not a uuid","battle_state_revision":0}),
            json!({"battle_uuid":"00000000-0000-0000-0000-000000000008"}),
            json!({"battle_uuid":null,"battle_state_revision":0}),
            json!({"battle_uuid":"00000000-0000-0000-0000-000000000008","battle_state_revision":-1}),
        ] {
            assert_eq!(
                code(resolved(&scenario, Some(&bad), 0)),
                "authoring_context_invalid"
            );
        }
        let scenario = parse(
            &input(
                json!({"type":"request_map_data"}),
                json!({"battle_event_after_seq":null}),
                json!([]),
            ),
            &ScenarioLimits::default(),
        );
        assert_eq!(
            code(resolved(&scenario, None, 0)),
            "authoring_context_invalid"
        );
        let scenario = parse(
            &input(json!({"type":"request_map_data"}), json!({}), json!([])),
            &ScenarioLimits::default(),
        );
        let result = resolved(&scenario, None, 0).unwrap().0;
        assert_eq!(result.context, ApplicationExecutionContext::default());
    }

    #[test]
    fn public_command_rejection_is_preserved_instead_of_a_successful_noop() {
        let document = input(
            json!({"type":"select_starter_employees", "loan_copy_ids":[], "employee_definition_ids":[null]}),
            json!({}),
            json!([{"from":{"step":0,"pointer":""},"to":"/command/employee_definition_ids/0","value_type":"number"}]),
        );
        let scenario = parse(&document, &ScenarioLimits::default());
        let error = resolved(&scenario, Some(&json!(7)), 0).unwrap_err();
        let expected = ApplicationCommand::from_document(ApplicationDocument::new(
            json!({"type":"select_starter_employees", "loan_copy_ids":[], "employee_definition_ids":[7]}),
        ))
        .unwrap_err();
        match error {
            AuthoringError::Application { phase, diagnostic } => {
                assert_eq!(phase, ExecutionPhase::TrialExecution);
                assert_eq!(diagnostic, expected);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn copied_bytes_and_resolved_bytes_have_independent_inclusive_caps() {
        let document = input(
            json!({"type":"request_map_data"}),
            json!({"source_command_id":null}),
            json!([string_binding("", "/context/source_command_id")]),
        );
        let source = json!("a\"b\\c\n");
        let copy_bytes = serde_json::to_vec(&source).unwrap().len();
        let resolved_bytes = serde_json::to_vec(
            &json!({"command":{"type":"request_map_data"},"context":{"source_command_id":source}}),
        )
        .unwrap()
        .len();
        let limits = ScenarioLimits {
            max_copy_bytes_per_step: copy_bytes,
            max_total_copy_bytes: copy_bytes + 3,
            max_resolved_bytes: resolved_bytes,
            ..Default::default()
        };
        let scenario = parse(&document, &limits);
        assert_eq!(scenario.limits().max_resolved_bytes, resolved_bytes);
        assert_eq!(
            resolved(&scenario, Some(&source), 3)
                .unwrap()
                .0
                .total_copy_bytes,
            copy_bytes + 3
        );
        assert_eq!(
            code(resolved(&scenario, Some(&source), 4)),
            "authoring_binding_copy_limit"
        );
        let smaller = parse(
            &document,
            &ScenarioLimits {
                max_resolved_bytes: resolved_bytes - 1,
                ..limits.clone()
            },
        );
        assert_eq!(
            code(resolved(&smaller, Some(&source), 3)),
            "authoring_resolved_byte_limit"
        );
        let smaller = parse(
            &document,
            &ScenarioLimits {
                max_copy_bytes_per_step: copy_bytes - 1,
                ..limits
            },
        );
        assert_eq!(
            code(resolved(&smaller, Some(&source), 3)),
            "authoring_resolved_byte_limit"
        );
    }

    #[test]
    fn copied_shape_is_checked_at_destination_before_context_or_command_decode() {
        let document = input(
            json!({"type":"request_map_data"}),
            json!({"expected_battle_authority":null}),
            json!([{"from":{"step":0,"pointer":""},"to":"/context/expected_battle_authority","value_type":"object"}]),
        );
        // The request itself fits depth 8. This borrowed source fits at root but
        // exceeds depth 8 after insertion at depth 2.
        let limits = ScenarioLimits {
            max_depth: 8,
            ..Default::default()
        };
        let scenario = parse(&document, &limits);
        let mut source = Value::Null;
        for _ in 0..7 {
            source = json!({"x":source});
        }
        assert_eq!(
            code(resolved(&scenario, Some(&source), 0)),
            "authoring_resolved_depth_limit"
        );
        let scenario = parse(
            &document,
            &ScenarioLimits {
                max_nodes: 50,
                ..Default::default()
            },
        );
        let source = json!({"many":vec![0;51]});
        assert_eq!(
            code(resolved(&scenario, Some(&source), 0)),
            "authoring_resolved_node_limit"
        );
    }

    #[test]
    fn cancellation_and_tight_work_preempt_decode_and_leave_retry_basis_intact() {
        let document = input(
            json!({"type":"request_map_data"}),
            json!({"source_command_id":null}),
            json!([string_binding("", "/context/source_command_id")]),
        );
        let scenario = parse(&document, &ScenarioLimits::default());
        let source = json!("stable");
        let (expected, usage) = resolved(&scenario, Some(&source), 2).unwrap();
        let token = CancellationToken::default();
        token.cancel();
        assert_eq!(
            code(run(
                &scenario,
                &[Some(&source)],
                2,
                &ExecutionLimits::default(),
                &token
            )),
            "authoring_cancelled"
        );
        let tight = ExecutionLimits {
            max_work: usage.work - 1,
            ..Default::default()
        };
        assert_eq!(
            code(run(
                &scenario,
                &[Some(&source)],
                2,
                &tight,
                &CancellationToken::default()
            )),
            "authoring_work_limit"
        );
        let (retry, retried_usage) = resolved(&scenario, Some(&source), 2).unwrap();
        assert_eq!(retried_usage, usage);
        assert_eq!(retry.command.document(), expected.command.document());
        assert_eq!(retry.total_copy_bytes, expected.total_copy_bytes);
        assert_eq!(
            code(resolve_command(&scenario, 1, &[Some(&source)], 2)),
            "authoring_scope_required"
        );
    }

    #[test]
    fn whole_resolved_node_count_includes_template_and_all_inserted_values() {
        let document = input(
            json!({"type":"select_starter_employees", "loan_copy_ids":[], "employee_definition_ids":null}),
            json!({}),
            json!([{"from":{"step":0,"pointer":""},"to":"/command/employee_definition_ids","value_type":"array"}]),
        );
        let source = json!(vec!["employee"; 46]);
        let exact = ScenarioLimits {
            max_nodes: 52,
            ..Default::default()
        };
        let scenario = parse(&document, &exact);
        resolved(&scenario, Some(&source), 0).unwrap();
        let scenario = parse(
            &document,
            &ScenarioLimits {
                max_nodes: 51,
                ..exact
            },
        );
        assert_eq!(
            code(resolved(&scenario, Some(&source), 0)),
            "authoring_resolved_node_limit"
        );
    }

    fn assertion_case(
        pointer: &str,
        assertion: Value,
        source: Option<&Value>,
        limits: &ScenarioLimits,
    ) -> Result<(AssertionObservation, ExecutionUsage), AuthoringError> {
        let mut document = input(json!({"type":"request_map_data"}), json!({}), json!([]));
        document["steps"][0] =
            json!({"kind":"assert", "from":{"step":0,"pointer":pointer},"assertion":assertion});
        let scenario = parse(&document, limits);
        let (result, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || resolve_assertion(&scenario, 1, &[source]),
        )
        .map_err(|diagnostic| AuthoringError::Execution { diagnostic })?;
        result.map(|value| (value, usage))
    }

    #[test]
    fn assertions_distinguish_absent_result_missing_path_null_and_type() {
        use super::super::JsonValueType as K;
        let limits = ScenarioLimits::default();
        let source = json!({"null":null, "array":[false], "01":true});
        let exists_false = json!({"test":"exists", "expected":false});
        assert_eq!(
            code(assertion_case(
                "/missing",
                exists_false.clone(),
                None,
                &limits
            )),
            "authoring_binding_source_unavailable"
        );
        assert_eq!(
            assertion_case("/missing", exists_false.clone(), Some(&source), &limits)
                .unwrap()
                .0,
            AssertionObservation {
                exists: false,
                value_type: None
            }
        );
        assert_eq!(
            code(assertion_case(
                "/null",
                exists_false.clone(),
                Some(&source),
                &limits
            )),
            "authoring_assertion_failed"
        );
        assert_eq!(
            assertion_case(
                "/null",
                json!({"test":"equals","value":null}),
                Some(&source),
                &limits
            )
            .unwrap()
            .0,
            AssertionObservation {
                exists: true,
                value_type: Some(K::Null)
            }
        );
        assert_eq!(
            code(assertion_case(
                "/missing",
                json!({"test":"equals","value":null}),
                Some(&source),
                &limits
            )),
            "authoring_assertion_failed"
        );
        for token in ["01", "-", "+1", "-1", "", "١", "184467440737095516160"] {
            assert_eq!(
                code(assertion_case(
                    &format!("/array/{token}"),
                    exists_false.clone(),
                    Some(&source),
                    &limits
                )),
                "authoring_pointer_invalid"
            );
        }
        assert_eq!(
            assertion_case("/array/1", exists_false, Some(&source), &limits)
                .unwrap()
                .0
                .exists,
            false
        );
        assert_eq!(
            assertion_case(
                "/01",
                json!({"test":"type","value_type":"bool"}),
                Some(&source),
                &limits
            )
            .unwrap()
            .0
            .value_type,
            Some(K::Bool)
        );
        assert_eq!(
            code(assertion_case(
                "/array",
                json!({"test":"type","value_type":"object"}),
                Some(&source),
                &limits
            )),
            "authoring_assertion_failed"
        );
    }

    #[test]
    fn assertion_equality_is_bounded_and_uses_json_semantics_without_copying_a_result() {
        let source = json!({"a":[1,2], "b":"value"});
        let assertion = json!({"test":"equals","value":{"b":"value","a":[1,2]}});
        let limits = ScenarioLimits::default();
        let (result, usage) =
            assertion_case("", assertion.clone(), Some(&source), &limits).unwrap();
        assert!(result.exists);
        assert!(usage.work > serde_json::to_vec(&source).unwrap().len() as u64);
        assert_eq!(
            assertion_case("", assertion.clone(), Some(&source), &limits)
                .unwrap()
                .1,
            usage
        );
        let limits = ScenarioLimits {
            max_resolved_bytes: 1,
            ..limits
        };
        assert_eq!(
            code(assertion_case("", assertion, Some(&source), &limits)),
            "authoring_resolved_byte_limit"
        );
        // Existence/type observe no large subtree and do not copy it into a new
        // public result. Their scalar observation remains valid at this cap.
        assert!(
            assertion_case(
                "",
                json!({"test":"exists","expected":true}),
                Some(&source),
                &limits
            )
            .unwrap()
            .0
            .exists
        );
    }

    #[test]
    fn preparation_control_failure_keeps_the_actual_step_phase() {
        let mut document = input(
            json!({"type":"request_map_data"}),
            json!({"source_command_id":null}),
            json!([string_binding("", "/context/source_command_id")]),
        );
        let command = document["steps"][0].take();
        document["steps"] = json!([]);
        document["preparation_steps"]
            .as_array_mut()
            .unwrap()
            .push(command);
        let scenario = parse(&document, &ScenarioLimits::default());
        let source = json!("id");
        let result = run(
            &scenario,
            &[Some(&source)],
            0,
            &ExecutionLimits {
                max_work: 1,
                ..Default::default()
            },
            &CancellationToken::default(),
        );
        match result.unwrap_err() {
            AuthoringError::Execution { diagnostic } => {
                assert_eq!(diagnostic.code, "authoring_work_limit");
                assert_eq!(diagnostic.phase, ExecutionPhase::TrialPreparation);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
