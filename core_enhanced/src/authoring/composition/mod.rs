//! Named, typed composition compiled into the existing Skill/Flow executor.
//! Definitions never own runtime state, and a source map never grants execution.
mod artifact;
mod budget;
mod compile;
mod predicates;
mod run_service;
#[cfg(test)]
pub(crate) use run_service::hit_counter_response;
mod schema;
mod service;
mod source;
mod trial;
pub use run_service::{
    run_composition, CompositionRunRequest, CompositionRunResponse, MAX_RUN_REQUEST_BYTES,
};
pub use trial::{CompositionRunIdentity, CompositionTrial, CompositionTrialFailure};

pub use artifact::Artifact;
pub use compile::compile_json;
pub use predicates::{
    PredicateBody, PredicateContext, PredicateDefinition, PredicateParameter, PredicateRange,
    PredicateType,
};
pub use schema::*;
use serde::{Deserialize, Serialize};
pub use service::{
    compile_request, CompileOutcome, CompileRequest, CompileResponse, SOURCE_TARGET,
};
pub use source::{compile_yaml, parse_yaml};
pub(crate) use source::compile_yaml_with_nesting;

pub const VERSION: u32 = 2;
pub const MAX_INPUT_BYTES: usize = 2_000_000;
pub const MAX_ARTIFACT_BYTES: usize = 8_000_000;
pub const MAX_CALL_DEPTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceTrace {
    /// JSON model path, independent of generated runtime addresses.
    pub definition: String,
    /// Outermost to innermost invocation, including the skill entry.
    pub calls: Vec<String>,
    /// Argument uses, forwarding sites and shared value declarations.
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DiagnosticCode {
    Parse,
    Version,
    Reference,
    Type,
    Unresolved,
    Unsupported,
    Limit,
    Native,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub document: String,
    pub source_digest: String,
    pub source: SourceTrace,
    pub reason: String,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {}: {}",
            self.code, self.document, self.source.definition, self.reason
        )
    }
}
impl std::error::Error for Diagnostic {}

#[cfg(test)]
mod action_modifier_tests;
#[cfg(test)]
pub(crate) mod canonical_area_tests;
#[cfg(test)]
pub(crate) mod canonical_deferred_tests;
#[cfg(test)]
mod multihit_tests;
#[cfg(test)]
mod source_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod predicate_tests;

#[cfg(test)]
pub(crate) mod owned_action_tests;

#[cfg(test)]
pub(crate) mod canonical_knight_tests;

#[cfg(test)]
pub(crate) mod canonical_hatred_tests;

#[cfg(test)]
pub(crate) mod canonical_pair_tests;

#[cfg(test)]
pub(crate) mod canonical_input_plan_tests;

#[cfg(test)]
pub(crate) mod canonical_little_red_equipment_tests;

#[cfg(test)]
pub(crate) mod canonical_wolf_equipment_tests;

#[cfg(test)]
pub(crate) mod tile_burst_tests;

#[cfg(test)]
pub(crate) mod canonical_funeral_tests;

#[cfg(test)]
pub(crate) mod presence_tests;

#[cfg(test)]
pub(crate) mod canonical_presence_tests;

#[cfg(test)]
pub(crate) mod canonical_channel_tests;

#[cfg(test)]
pub(crate) mod overlay_tests;

#[cfg(test)]
pub(crate) mod canonical_overlay_tests;

#[cfg(test)]
pub(crate) mod canonical_mountain_tests;

#[cfg(test)]
pub(crate) mod canonical_greed_tests;
