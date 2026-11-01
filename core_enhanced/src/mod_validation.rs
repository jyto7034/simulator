//! Issuer-owned validation of immutable mod selections. Static content validation
//! remains diagnostic; only a completed, calibrated Lab can issue an approval.
mod cache;
mod candidate;
pub(crate) mod coverage;
mod error;
mod host;
mod identity;
mod issuer;
mod lab;
mod schema;

#[cfg(test)]
#[path = "../docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/shared/cache_hooks.rs"]
pub(crate) mod u11_cache_hooks;
#[cfg(test)]
#[path = "../docs/goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/shared/cache_tests.rs"]
mod u11_cache_tests;

pub use candidate::{Candidate, CandidateView, TrustedBase};
pub use coverage::{coverage_support, CoverageSupport, DefinitionCoverageSupport, COVERAGE_SUPPORT_REVISION, CoverageKey, CoveragePlan, CoverageReport, CoverageRequirement, CoverageStage};
pub use error::{ModValidationError, ModValidationErrorKind};
pub use host::{ApprovedHost, HostApplicationError, HostSelectionView};
pub use identity::{BuildIdentity, ExecutionPairIdentity};
pub use issuer::{
    ApprovedGeneration, CancelDisposition, JobId, JobState, JobView, ValidationIssuer,
};
pub use lab::{
    execute_worker, run_driver, CalibrationRun, CriterionAssessment, CriterionLimits, DriverReport,
    LabCase, LabCaseReport, LabRemovedReference, RemovedReferenceReport, LabCriterion, LabMetrics, LabOutcome, LabPhaseSample, LabProfile,
    LabRun, LabStatus, LabSuite, WorkerMeasurement, WorkerMode, WorkerPair, WorkerReport,
    WorkerRequest, WorkerResponse, WorkerFailure, WorkloadClass,
};
pub use schema::{ModDefinition, ModRegistry, ModSelection, RequiredConfiguration, RequiredPack};

pub(crate) fn digest(value: &impl serde::Serialize) -> Result<String, ModValidationError> {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(value)
        .map_err(|e| ModValidationError::invalid("identity", e.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
