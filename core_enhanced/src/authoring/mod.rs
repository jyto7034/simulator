//! Common content-authoring operations over the production loader and core.

mod amount;
mod provenance;
pub mod composition;
pub mod node_graph;
mod amount_query;
#[cfg(test)]
pub(crate) use amount_query::{encode_record_for_test, validate_record_generation_for_test};
pub(crate) mod clone_shape;
mod compiled;
pub(crate) mod control;
mod definition;
mod lab_preparation;
pub use lab_preparation::{LabEmployeeGroup,LabRelicLoadout,LabPlacement,LabPreparation,LabPreparationReceipt,WorldLabPreparation};
mod diagnostic;
mod output;
mod preview;
pub(crate) mod projection;
mod query;
mod resolve;
mod scenario;
pub(crate) mod trace;
mod trial;
mod view;

pub use amount_query::{AmountBattle, AmountQuery};
pub use diagnostic::{
    render_error, render_trial_failure, DiagnosticLimits, DiagnosticRenderError, DiagnosticUsage,
    RenderedDiagnostic,
};
pub use preview::TrialPreview;
pub use query::{describe, explain, validation_receipt, AuthoringQueryResult, ExplainQuery};
pub use scenario::{
    Assertion, Binding, JsonValueType, ResultPointer, ScenarioLimits, TrialClock, TrialInitial,
    TrialProfile, TrialScenario, TrialStep,
};
pub use trace::{TraceGap, TraceGapReason, TraceLimits, TraceOptions};
pub use trial::{
    simulate, ReceiptView, SimulationFailure, StepView, TrialFailure, TrialLimits, TrialPosition,
    TrialSession,
};
pub use view::{
    ApplicationChange, ApplicationView, AreaChange, AreaMembership, AreaWitness, AuthoredEffectView, BattleKey, BattleLifecycleView,
    BattleObservationView, BattleTraceView, BuffApplicationState, EffectSourceView, LifecycleView,
    ObservationView, ProcOriginView, SessionTraceView, TerminalObservationView,
};

use crate::game::data::{
    content_pack::{self, ContentSelection, InputLimits},
    live_ron::LiveRonError,
    ValidatedGameData,
};
use serde::Serialize;
use std::{fmt, path::Path};

pub use control::{
    CancellationToken, ExecutionFailure, ExecutionLimits, ExecutionPhase, ExecutionUsage,
};

#[derive(Debug, Clone)]
pub struct AuthoringLimits {
    pub input: InputLimits,
    pub execution: ExecutionLimits,
}
impl AuthoringLimits {
    pub fn with_runtime_safety(settings: &crate::runtime_safety::RuntimeSafetySettings) -> Self {
        Self {
            input: InputLimits {
                value_expr: settings.expression_parse(),
                ..InputLimits::default()
            },
            execution: settings.authoring_execution(),
        }
    }
}
impl Default for AuthoringLimits {
    fn default() -> Self {
        Self {
            input: InputLimits::default(),
            execution: ExecutionLimits::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthoringError {
    Content {
        diagnostic: LiveRonError,
    },
    Execution {
        diagnostic: ExecutionFailure,
    },
    Request {
        diagnostic: RequestFailure,
    },
    Application {
        phase: ExecutionPhase,
        diagnostic: crate::application::ApplicationError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequestFailure {
    pub code: &'static str,
    pub phase: ExecutionPhase,
    pub field_path: &'static str,
    pub reason: String,
}
impl fmt::Display for AuthoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Content { diagnostic } => diagnostic.fmt(f),
            Self::Execution { diagnostic } => diagnostic.fmt(f),
            Self::Request { diagnostic } => write!(
                f,
                "{} at {}: {}",
                diagnostic.code, diagnostic.field_path, diagnostic.reason
            ),
            Self::Application { diagnostic, .. } => write!(f, "{}", diagnostic.code()),
        }
    }
}
impl std::error::Error for AuthoringError {}

/// A successfully validated external generation. The token and its explanation
/// metadata come from one publication and cannot be replaced independently.
pub struct AuthoringContent {
    token: ValidatedGameData,
    validation_usage: ExecutionUsage,
}
impl AuthoringContent {
    pub(crate) fn from_candidate(token: ValidatedGameData, validation_usage: ExecutionUsage) -> Self {
        Self { token, validation_usage }
    }
    pub fn token(&self) -> &ValidatedGameData {
        &self.token
    }
    pub fn validation_usage(&self) -> &ExecutionUsage {
        &self.validation_usage
    }
}

/// Validate explicit external content using the production loader. Limits are
/// trusted settings with finite ceilings; cancellation is cooperative.
pub fn validate(
    root: &Path,
    selection: &ContentSelection,
    limits: &AuthoringLimits,
    cancellation: &CancellationToken,
) -> Result<AuthoringContent, AuthoringError> {
    validate_internal(
        root,
        selection,
        limits,
        cancellation,
        crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
    )
}

/// Profile-aware admission. The profile owns execution and expression limits;
/// the caller supplies the separate acquisition/input family. The legacy
/// `validate` API continues to accept explicit typed execution limits.
pub fn validate_with_runtime_safety(
    root: &Path,
    selection: &ContentSelection,
    input: &InputLimits,
    cancellation: &CancellationToken,
    settings: std::sync::Arc<crate::runtime_safety::RuntimeSafetySettings>,
) -> Result<AuthoringContent, AuthoringError> {
    let limits = AuthoringLimits {
        input: input.clone(),
        execution: settings.authoring_execution(),
    };
    validate_internal(root, selection, &limits, cancellation, settings)
}

fn validate_internal(
    root: &Path,
    selection: &ContentSelection,
    limits: &AuthoringLimits,
    cancellation: &CancellationToken,
    settings: std::sync::Arc<crate::runtime_safety::RuntimeSafetySettings>,
) -> Result<AuthoringContent, AuthoringError> {
    let hard = InputLimits::default();
    for (site, requested, ceiling) in [
        ("max_packs", limits.input.max_packs, hard.max_packs),
        ("max_files", limits.input.max_files, hard.max_files),
        (
            "max_file_bytes",
            limits.input.max_file_bytes,
            hard.max_file_bytes,
        ),
        (
            "max_total_bytes",
            limits.input.max_total_bytes,
            hard.max_total_bytes,
        ),
        (
            "max_dependency_edges",
            limits.input.max_dependency_edges,
            hard.max_dependency_edges,
        ),
        ("max_nesting", limits.input.max_nesting, hard.max_nesting),
        (
            "value_expr.max_input_bytes",
            limits.input.value_expr.max_input_bytes,
            hard.value_expr.max_input_bytes,
        ),
        (
            "value_expr.max_depth",
            limits.input.value_expr.max_depth,
            hard.value_expr.max_depth,
        ),
        (
            "value_expr.max_nodes",
            limits.input.value_expr.max_nodes,
            hard.value_expr.max_nodes,
        ),
        (
            "value_expr.max_authored_members",
            limits.input.value_expr.max_authored_members,
            hard.value_expr.max_authored_members,
        ),
    ] {
        if requested == 0 || requested > ceiling {
            return Err(AuthoringError::Execution {
                diagnostic: ExecutionFailure {
                    code: "authoring_limits_invalid",
                    phase: ExecutionPhase::Acquisition,
                    site,
                    limit: Some(ceiling as u64),
                    requested: Some(requested as u64),
                },
            });
        }
    }
    let (result, validation_usage) = control::run(
        &limits.execution,
        cancellation,
        ExecutionPhase::Acquisition,
        || content_pack::load_with_runtime_safety(root, selection, &limits.input, settings),
    )
    .map_err(|diagnostic| AuthoringError::Execution { diagnostic })?;
    let token = result.map_err(|diagnostic| AuthoringError::Content { diagnostic })?;
    Ok(AuthoringContent {
        token,
        validation_usage,
    })
}

#[cfg(all(test, unix))]
mod tests;
