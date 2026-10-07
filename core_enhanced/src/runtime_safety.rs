//! Immutable trusted-host safety settings. This module never reads files or environment.
//! Numeric defaults have one source: the embedded deployment RON. Engineering ceilings
//! remain owned by the engines. An explicit trusted measurement profile may
//! select a finite retained window without changing deployment validation.
use crate::{
    authoring::{ExecutionLimits, ExecutionPhase},
    game::{
        battle::runtime_control::RuntimeFactLimits,
        value_expr::{ExprLimits, ExprParseLimits},
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    sync::{Arc, OnceLock},
};

pub const MAX_SETTINGS_BYTES: usize = 64 * 1024;
const MAX_SETTINGS_DEPTH: usize = 16;
pub const EMBEDDED_RUNTIME_SAFETY_RON: &[u8] = include_bytes!("../../config/runtime_safety.ron");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSafetyError {
    pub source: String,
    pub field: String,
    pub message: String,
}
impl fmt::Display for RuntimeSafetyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.source, self.field, self.message)
    }
}
impl std::error::Error for RuntimeSafetyError {}
fn error(source: &str, field: impl Into<String>, message: impl Into<String>) -> RuntimeSafetyError {
    RuntimeSafetyError {
        source: source.into(),
        field: field.into(),
        message: message.into(),
    }
}

/// A diagnostic retention window, never a limit on battle duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BattleHistoryLimits {
    pub max_events: usize,
    pub max_encoded_entry_array_bytes: usize,
    pub max_span_ms: u64,
}
crate::authoring::clone_shape::copy_shape!(BattleHistoryLimits);
impl Default for BattleHistoryLimits {
    fn default() -> Self {
        RuntimeSafetySettings::embedded().battle_history()
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeSafetySettings {
    native_runtime: RuntimeFactLimits,
    development_measurement: Option<DevelopmentMeasurementProfile>,
    authoring_execution: ExecutionLimits,
    expression_parse: ExprParseLimits,
    expression: ExprLimits,
    trial_work: u64,
    trial_receipt_bytes: usize,
    history: BattleHistoryLimits,
    raw_sha256: String,
    effective_sha256: String,
}

/// Finite trusted measurement window, distinct from deployment limits.
#[derive(Debug, Clone, Copy)]
pub struct DevelopmentMeasurementProfile {
    revision: u32,
    emergency_retained_bytes: usize,
}

impl DevelopmentMeasurementProfile {
    pub fn revision(self) -> u32 {
        self.revision
    }

    pub fn emergency_retained_bytes(self) -> usize {
        self.emergency_retained_bytes
    }
}

pub const DEVELOPMENT_RETAINED_CEILING: usize = 32 * 1024 * 1024;

impl RuntimeSafetySettings {
    /// The same validated RON backs no-file embedding and each engine's Default.
    pub fn embedded() -> &'static Self {
        static DEFAULT: OnceLock<RuntimeSafetySettings> = OnceLock::new();
        DEFAULT.get_or_init(|| {
            Self::from_ron("embedded runtime_safety.ron", EMBEDDED_RUNTIME_SAFETY_RON)
                .expect("embedded runtime safety settings must satisfy engine ceilings")
        })
    }
    pub fn embedded_shared() -> Arc<Self> {
        static DEFAULT: OnceLock<Arc<RuntimeSafetySettings>> = OnceLock::new();
        Arc::clone(DEFAULT.get_or_init(|| Arc::new(Self::embedded().clone())))
    }
    pub fn native_runtime(&self) -> RuntimeFactLimits {
        let mut limits = self.native_runtime;
        if let Some(profile) = self.development_measurement {
            limits.max_retained_bytes = profile.emergency_retained_bytes;
        }
        limits
    }
    /// Nominal deployment limit also serves as the measurement warning line.
    pub fn nominal_native_runtime(&self) -> RuntimeFactLimits {
        self.native_runtime
    }
    pub fn development_measurement(&self) -> Option<DevelopmentMeasurementProfile> {
        self.development_measurement
    }
    pub fn authoring_execution(&self) -> ExecutionLimits {
        self.authoring_execution.clone()
    }
    pub fn expression_parse(&self) -> ExprParseLimits {
        self.expression_parse
    }
    pub fn expression(&self) -> ExprLimits {
        self.expression
    }
    pub fn trial_max_total_work(&self) -> u64 {
        self.trial_work
    }
    pub fn trial_max_receipt_bytes(&self) -> usize {
        self.trial_receipt_bytes
    }
    pub fn battle_history(&self) -> BattleHistoryLimits {
        self.history
    }
    pub fn verify_effective_sha256(&self, expected: &str) -> Result<(), RuntimeSafetyError> {
        if expected == self.effective_sha256 {
            Ok(())
        } else {
            Err(error(
                "replay",
                "runtime_safety_sha256",
                format!(
                    "profile mismatch: expected {expected}, actual {}",
                    self.effective_sha256
                ),
            ))
        }
    }
    pub fn raw_sha256(&self) -> &str {
        &self.raw_sha256
    }
    pub fn effective_sha256(&self) -> &str {
        &self.effective_sha256
    }

    /// Reject bounded syntax/schema/values before content publication or session creation.
    /// Hosts supply their selected path as `source` for diagnostics, and must bound file I/O.
    pub fn from_ron(source: &str, bytes: &[u8]) -> Result<Self, RuntimeSafetyError> {
        guard_syntax(source, bytes)?;
        let mut deserializer = ron::de::Deserializer::from_bytes(bytes)
            .map_err(|e| error(source, "$", e.to_string()))?;
        let dto: SettingsDto =
            serde_path_to_error::deserialize(&mut deserializer).map_err(|e| {
                let mut path = e.path().to_string();
                // RON 0.5 carries Serde structural diagnostics as messages.
                // Serde reports these at the parent object, so retain its
                // field name as well as serde_path_to_error's parent path.
                let message = e.inner().to_string();
                let field = ["unknown field `", "missing field `", "duplicate field `"]
                    .iter()
                    .find_map(|prefix| message.strip_prefix(prefix))
                    .and_then(|suffix| suffix.split('`').next());
                if let Some(field) = field {
                    if path == "." || path.is_empty() {
                        path = field.into();
                    } else if path.rsplit('.').next() != Some(field) {
                        path.push('.');
                        path.push_str(field);
                    }
                }
                error(source, path, e.inner().to_string())
            })?;
        deserializer
            .end()
            .map_err(|e| error(source, "$", e.to_string()))?;
        if dto.schema_version != 1 {
            return Err(error(source, "schema_version", "expected schema version 1"));
        }
        let native_runtime = dto.native_runtime.typed(source)?;
        native_runtime.validate().map_err(|e| match e {
            crate::game::behavior::GameError::BattleRuntimeFault(fault) => error(
                source,
                format!("native_runtime.{}", fault.site),
                format!(
                    "outside supported nonzero range (ceiling {:?})",
                    fault.limit
                ),
            ),
            other => error(source, "native_runtime", format!("{other:?}")),
        })?;
        let development_measurement = dto
            .development_measurement
            .as_ref()
            .map(|profile| {
                if profile.revision != 1 {
                    return Err(error(
                        source,
                        "development_measurement.revision",
                        "expected revision 1",
                    ));
                }
                bounded(
                    source,
                    "development_measurement.emergency_retained_bytes",
                    profile.emergency_retained_bytes,
                    DEVELOPMENT_RETAINED_CEILING as u64,
                )?;
                let emergency_retained_bytes = count(
                    source,
                    "development_measurement.emergency_retained_bytes",
                    profile.emergency_retained_bytes,
                )?;
                if emergency_retained_bytes <= native_runtime.max_retained_bytes {
                    return Err(error(
                        source,
                        "development_measurement.emergency_retained_bytes",
                        "must exceed the nominal warning limit",
                    ));
                }
                Ok(DevelopmentMeasurementProfile {
                    revision: 1,
                    emergency_retained_bytes,
                })
            })
            .transpose()?;
        let authoring_execution = dto.authoring_execution.typed(source)?;
        authoring_execution
            .validate(ExecutionPhase::Acquisition)
            .map_err(|e| {
                error(
                    source,
                    format!("authoring_execution.{}", e.site),
                    format!("outside supported nonzero range (ceiling {:?})", e.limit),
                )
            })?;
        let expression_parse = dto.expression.parse.typed(source)?;
        for (name, value, cap) in [
            (
                "max_input_bytes",
                expression_parse.max_input_bytes,
                ExprParseLimits::HARD_MAX.max_input_bytes,
            ),
            (
                "max_depth",
                expression_parse.max_depth,
                ExprParseLimits::HARD_MAX.max_depth,
            ),
            (
                "max_nodes",
                expression_parse.max_nodes,
                ExprParseLimits::HARD_MAX.max_nodes,
            ),
            (
                "max_authored_members",
                expression_parse.max_authored_members,
                ExprParseLimits::HARD_MAX.max_authored_members,
            ),
        ] {
            bounded(
                source,
                &format!("expression.parse.{name}"),
                value as u64,
                cap as u64,
            )?;
        }
        let expression = dto.expression.compile_and_evaluate.typed(source)?;
        // ExprLimits intentionally has no new upper ceiling: preserve its typed API range.
        let trial_work = dto.authoring_trial.max_total_work;
        bounded(
            source,
            "authoring_trial.max_total_work",
            trial_work,
            1 << 38,
        )?;
        bounded(
            source,
            "authoring_trial.max_receipt_bytes",
            dto.authoring_trial.max_receipt_bytes,
            64 * 1024 * 1024,
        )?;
        let trial_receipt_bytes = count(
            source,
            "authoring_trial.max_receipt_bytes",
            dto.authoring_trial.max_receipt_bytes,
        )?;
        let h = &dto.battle_history;
        bounded(
            source,
            "battle_history.max_events",
            h.max_events,
            crate::game::battle::event_log::BATTLE_HISTORY_MAX_EVENTS as u64,
        )?;
        bounded(
            source,
            "battle_history.max_encoded_entry_array_bytes",
            h.max_encoded_entry_array_bytes,
            crate::game::battle::event_log::BATTLE_HISTORY_MAX_ENCODED_BYTES as u64,
        )?;
        bounded(
            source,
            "battle_history.max_span_ms",
            h.max_span_ms,
            crate::game::battle::event_log::BATTLE_HISTORY_MAX_SPAN_MS,
        )?;
        if h.max_encoded_entry_array_bytes < 2 {
            return Err(error(
                source,
                "battle_history.max_encoded_entry_array_bytes",
                "must fit an empty JSON entry array (2 bytes)",
            ));
        }
        let history = BattleHistoryLimits {
            max_events: count(source, "battle_history.max_events", h.max_events)?,
            max_encoded_entry_array_bytes: count(
                source,
                "battle_history.max_encoded_entry_array_bytes",
                h.max_encoded_entry_array_bytes,
            )?,
            max_span_ms: h.max_span_ms,
        };
        let effective = serde_json::to_vec(&dto).expect("fixed integer DTO serializes infallibly");
        Ok(Self {
            native_runtime,
            development_measurement,
            authoring_execution,
            expression_parse,
            expression,
            trial_work,
            trial_receipt_bytes,
            history,
            raw_sha256: digest(bytes),
            effective_sha256: digest(&effective),
        })
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn count(source: &str, field: &str, value: u64) -> Result<usize, RuntimeSafetyError> {
    usize::try_from(value)
        .map_err(|_| error(source, field, "value exceeds this target's usize width"))
}
fn bounded(source: &str, field: &str, value: u64, ceiling: u64) -> Result<(), RuntimeSafetyError> {
    if value == 0 || value > ceiling {
        Err(error(
            source,
            field,
            format!("expected 1..={ceiling}, received {value}"),
        ))
    } else {
        Ok(())
    }
}

// Bound nesting before entering RON (including nested block comments). Strings are
// lexically skipped but the integer-only DTO rejects them. No recursive scanner.
fn guard_syntax(source: &str, bytes: &[u8]) -> Result<(), RuntimeSafetyError> {
    if bytes.len() > MAX_SETTINGS_BYTES {
        return Err(error(source, "$", "settings exceed 64KiB"));
    }
    let (mut i, mut depth, mut comment_depth) = (0, 0usize, 0usize);
    let (mut quoted, mut line_comment) = (false, false);
    while i < bytes.len() {
        let c = bytes[i];
        let next = bytes.get(i + 1).copied();
        if line_comment {
            if c == b'\n' {
                line_comment = false;
            }
            i += 1;
            continue;
        }
        if comment_depth > 0 {
            if c == b'/' && next == Some(b'*') {
                comment_depth += 1;
                i += 2;
            } else if c == b'*' && next == Some(b'/') {
                comment_depth -= 1;
                i += 2;
            } else {
                i += 1;
            }
            if comment_depth > MAX_SETTINGS_DEPTH {
                return Err(error(
                    source,
                    "$",
                    "settings comment nesting exceeds fixed parser limit",
                ));
            }
            continue;
        }
        if quoted {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                quoted = false;
            }
            i += 1;
            continue;
        }
        if c == b'/' && next == Some(b'/') {
            line_comment = true;
            i += 2;
            continue;
        }
        if c == b'/' && next == Some(b'*') {
            comment_depth = 1;
            i += 2;
            continue;
        }
        if c == b'"' {
            quoted = true;
        } else if matches!(c, b'(' | b'[' | b'{') {
            depth += 1;
            if depth > MAX_SETTINGS_DEPTH {
                return Err(error(
                    source,
                    "$",
                    "settings nesting exceeds fixed parser limit",
                ));
            }
        } else if matches!(c, b')' | b']' | b'}') {
            depth = depth.saturating_sub(1);
        }
        i += 1;
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsDto {
    schema_version: u64,
    native_runtime: NativeDto,
    authoring_execution: ExecutionDto,
    expression: ExpressionSettingsDto,
    authoring_trial: TrialDto,
    battle_history: HistoryDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    development_measurement: Option<DevelopmentMeasurementDto>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DevelopmentMeasurementDto {
    revision: u64,
    emergency_retained_bytes: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpressionSettingsDto {
    parse: ParseDto,
    compile_and_evaluate: ExpressionDto,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrialDto {
    max_total_work: u64,
    max_receipt_bytes: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryDto {
    max_events: u64,
    max_encoded_entry_array_bytes: u64,
    max_span_ms: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeDto {
    max_work: u64,
    max_clone_slots: u64,
    max_active_actions: u64,
    max_retained_results: u64,
    max_deliveries: u64,
    max_actors: u64,
    max_providers: u64,
    max_control_steps: u64,
    max_strike_segments: u64,
    max_predeath_buffs: u64,
    max_retained_bytes: u64,
}
impl NativeDto {
    fn typed(&self, source: &str) -> Result<RuntimeFactLimits, RuntimeSafetyError> {
        Ok(RuntimeFactLimits {
            max_work: self.max_work,
            max_clone_slots: count(
                source,
                "native_runtime.max_clone_slots",
                self.max_clone_slots,
            )?,
            max_active_actions: count(
                source,
                "native_runtime.max_active_actions",
                self.max_active_actions,
            )?,
            max_retained_results: count(
                source,
                "native_runtime.max_retained_results",
                self.max_retained_results,
            )?,
            max_deliveries: count(source, "native_runtime.max_deliveries", self.max_deliveries)?,
            max_actors: count(source, "native_runtime.max_actors", self.max_actors)?,
            max_providers: count(source, "native_runtime.max_providers", self.max_providers)?,
            max_control_steps: count(
                source,
                "native_runtime.max_control_steps",
                self.max_control_steps,
            )?,
            max_strike_segments: count(
                source,
                "native_runtime.max_strike_segments",
                self.max_strike_segments,
            )?,
            max_predeath_buffs: count(
                source,
                "native_runtime.max_predeath_buffs",
                self.max_predeath_buffs,
            )?,
            max_retained_bytes: count(
                source,
                "native_runtime.max_retained_bytes",
                self.max_retained_bytes,
            )?,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionDto {
    max_work: u64,
    max_pending_events: u64,
    max_bucket_events: u64,
    max_effect_commands: u64,
    max_geometry_cells: u64,
    max_retained_entities: u64,
    max_definitions: u64,
    max_metadata_bytes: u64,
    max_result_bytes: u64,
    max_clone_slots: u64,
}
impl ExecutionDto {
    fn typed(&self, source: &str) -> Result<ExecutionLimits, RuntimeSafetyError> {
        Ok(ExecutionLimits {
            max_work: self.max_work,
            max_pending_events: count(
                source,
                "authoring_execution.max_pending_events",
                self.max_pending_events,
            )?,
            max_bucket_events: count(
                source,
                "authoring_execution.max_bucket_events",
                self.max_bucket_events,
            )?,
            max_effect_commands: count(
                source,
                "authoring_execution.max_effect_commands",
                self.max_effect_commands,
            )?,
            max_geometry_cells: count(
                source,
                "authoring_execution.max_geometry_cells",
                self.max_geometry_cells,
            )?,
            max_retained_entities: count(
                source,
                "authoring_execution.max_retained_entities",
                self.max_retained_entities,
            )?,
            max_definitions: count(
                source,
                "authoring_execution.max_definitions",
                self.max_definitions,
            )?,
            max_metadata_bytes: count(
                source,
                "authoring_execution.max_metadata_bytes",
                self.max_metadata_bytes,
            )?,
            max_result_bytes: count(
                source,
                "authoring_execution.max_result_bytes",
                self.max_result_bytes,
            )?,
            max_clone_slots: count(
                source,
                "authoring_execution.max_clone_slots",
                self.max_clone_slots,
            )?,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParseDto {
    max_input_bytes: u64,
    max_depth: u64,
    max_nodes: u64,
    max_authored_members: u64,
}
impl ParseDto {
    fn typed(&self, source: &str) -> Result<ExprParseLimits, RuntimeSafetyError> {
        Ok(ExprParseLimits {
            max_input_bytes: count(
                source,
                "expression.parse.max_input_bytes",
                self.max_input_bytes,
            )?,
            max_depth: count(source, "expression.parse.max_depth", self.max_depth)?,
            max_nodes: count(source, "expression.parse.max_nodes", self.max_nodes)?,
            max_authored_members: count(
                source,
                "expression.parse.max_authored_members",
                self.max_authored_members,
            )?,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpressionDto {
    max_depth: u64,
    max_nodes: u64,
    max_stack: u64,
    max_dynamic_members: u64,
    max_groups: u64,
    max_work: u64,
    max_compile_work: u64,
    max_retained_bytes: u64,
}
impl ExpressionDto {
    fn typed(&self, source: &str) -> Result<ExprLimits, RuntimeSafetyError> {
        bounded(
            source,
            "expression.compile_and_evaluate.max_depth",
            self.max_depth,
            u64::MAX,
        )?;
        bounded(
            source,
            "expression.compile_and_evaluate.max_nodes",
            self.max_nodes,
            u64::MAX,
        )?;
        bounded(
            source,
            "expression.compile_and_evaluate.max_stack",
            self.max_stack,
            u64::MAX,
        )?;
        bounded(
            source,
            "expression.compile_and_evaluate.max_dynamic_members",
            self.max_dynamic_members,
            u64::MAX,
        )?;
        bounded(
            source,
            "expression.compile_and_evaluate.max_groups",
            self.max_groups,
            u64::MAX,
        )?;
        bounded(
            source,
            "expression.compile_and_evaluate.max_work",
            self.max_work,
            u64::MAX,
        )?;
        bounded(
            source,
            "expression.compile_and_evaluate.max_compile_work",
            self.max_compile_work,
            u64::MAX,
        )?;
        bounded(
            source,
            "expression.compile_and_evaluate.max_retained_bytes",
            self.max_retained_bytes,
            u64::MAX,
        )?;
        Ok(ExprLimits {
            max_depth: count(
                source,
                "expression.compile_and_evaluate.max_depth",
                self.max_depth,
            )?,
            max_nodes: count(
                source,
                "expression.compile_and_evaluate.max_nodes",
                self.max_nodes,
            )?,
            max_stack: count(
                source,
                "expression.compile_and_evaluate.max_stack",
                self.max_stack,
            )?,
            max_dynamic_members: count(
                source,
                "expression.compile_and_evaluate.max_dynamic_members",
                self.max_dynamic_members,
            )?,
            max_groups: count(
                source,
                "expression.compile_and_evaluate.max_groups",
                self.max_groups,
            )?,
            max_work: self.max_work,
            max_compile_work: self.max_compile_work,
            max_retained_bytes: count(
                source,
                "expression.compile_and_evaluate.max_retained_bytes",
                self.max_retained_bytes,
            )?,
        })
    }
}

#[cfg(test)]
mod tests;
