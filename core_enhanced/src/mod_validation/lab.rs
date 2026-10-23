//! The same finite Core driver is consumed by diagnostic calls and by fresh
//! plain/instrumented worker processes. Only the issuer-owned supervisor can
//! produce an approvable paired result.
use super::{
    coverage::CoveragePlan, digest, BuildIdentity, Candidate, CoverageReport,
    ExecutionPairIdentity, ModValidationError, ModValidationErrorKind,
};
use crate::authoring::{
    self, CancellationToken, ScenarioLimits, TrialLimits, TrialScenario, TrialSession,
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadClass {
    Normal,
    Expanded,
    Pathological,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabCase {
    pub id: String,
    pub class: WorkloadClass,
    pub scenario: Vec<u8>,
    #[serde(default)]
    pub preparation: Option<authoring::LabPreparation>,
    #[serde(default)]
    pub removed_references: Vec<LabRemovedReference>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabRemovedReference {
    pub key: crate::game::data::content_pack::DefinitionKey,
    pub scenario: Vec<u8>,
    pub preparation: authoring::LabPreparation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemovedReferenceReport {
    pub key: crate::game::data::content_pack::DefinitionKey,
    pub base_generation: crate::game::data::content_pack::ContentIdentity,
    pub base_preparation: authoring::LabPreparationReceipt,
    pub rejected_reference: crate::application::ApplicationError,
    pub setup_work: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabSuite {
    pub revision: String,
    #[serde(default)]
    pub required_definitions: Vec<crate::game::data::content_pack::DefinitionKey>,
    pub cases: Vec<LabCase>,
}
impl LabSuite {
    pub(crate) fn validate(&self) -> Result<(), ModValidationError> {
        let mut names = std::collections::BTreeSet::new();
        if self.revision.is_empty()
            || self.revision.len() > 128
            || self.cases.is_empty()
            || self.cases.len() > 64
            || self.required_definitions.len() > 8192
            || self
                .required_definitions
                .iter()
                .any(|key| key.id.is_empty() || key.id.len() > 128)
        {
            return Err(ModValidationError::invalid(
                "suite",
                "finite suite/revision required",
            ));
        }
        for c in &self.cases {
            if c.id.is_empty()
                || c.id.len() > 128
                || !names.insert(&c.id)
                || c.scenario.len() > 8 * 1024 * 1024
                || c.removed_references.len() > 16
                || c.removed_references.iter().any(|reference| {
                    reference.key.id.is_empty()
                        || reference.key.id.len() > 128
                        || reference.scenario.len() > 8 * 1024 * 1024
                })
            {
                return Err(ModValidationError::invalid(
                    "suite.cases",
                    "invalid/duplicate/oversized case",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LabCriterion {
    pub revision: String,
    pub calibration_sha256: String,
    pub max_elapsed_ns: u64,
    pub max_requested_heap_bytes: u64,
    pub max_rss_bytes: u64,
    pub max_native_work: u64,
    #[serde(skip)]
    pub(crate) binding: CriterionBinding,
}
// Untrusted JSON cannot reconstitute proof of an actual supervised calibration.
impl<'de> Deserialize<'de> for LabCriterion {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom(
            "LabCriterion requires actual supervised calibration adoption",
        ))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CriterionBinding {
    context_sha256: String,
    sealed_sha256: String,
    actual: LabOutcome,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriterionLimits {
    pub max_elapsed_ns: u64,
    pub max_requested_heap_bytes: u64,
    pub max_rss_bytes: u64,
    pub max_native_work: u64,
}
/// Created only by the fresh-process supervisor. Owned metrics are observable;
/// their public DTO cannot be converted back into this object.
pub struct CalibrationRun {
    run: LabRun,
    context_sha256: String,
}
impl CalibrationRun {
    pub fn outcome(&self) -> &LabOutcome {
        self.run.outcome()
    }
    pub fn evidence_directory(&self) -> Option<&Path> {
        self.run.evidence_directory()
    }
    pub fn adopt(
        &self,
        revision: String,
        limits: CriterionLimits,
    ) -> Result<LabCriterion, ModValidationError> {
        if !self.run.metrics().is_complete() {
            return Err(ModValidationError::incomplete(
                "calibration.metrics",
                "actual complete paired measurements required",
            ));
        }
        let calibration_sha256 = digest(self.run.outcome())?;
        let mut criterion = LabCriterion {
            revision,
            calibration_sha256,
            max_elapsed_ns: limits.max_elapsed_ns,
            max_requested_heap_bytes: limits.max_requested_heap_bytes,
            max_rss_bytes: limits.max_rss_bytes,
            max_native_work: limits.max_native_work,
            binding: CriterionBinding {
                context_sha256: self.context_sha256.clone(),
                sealed_sha256: String::new(),
                actual: self.run.outcome().clone(),
            },
        };
        criterion.binding.sealed_sha256 = digest(&criterion)?;
        if !criterion.valid() {
            return Err(ModValidationError::invalid(
                "criterion",
                "finite versioned adopted limits required",
            ));
        }
        Ok(criterion)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CriterionWire {
    revision: String,
    calibration_sha256: String,
    limits: CriterionLimits,
    binding: CriterionBinding,
}
impl LabCriterion {
    fn valid(&self) -> bool {
        !self.revision.is_empty()
            && self.revision.len() <= 128
            && hex(&self.calibration_sha256)
            && [
                self.max_elapsed_ns,
                self.max_requested_heap_bytes,
                self.max_rss_bytes,
                self.max_native_work,
            ]
            .into_iter()
            .all(|n| n > 0)
            && self.binding.actual.metrics.is_complete()
            && digest(&self.binding.actual).is_ok_and(|s| s == self.calibration_sha256)
            && digest(self).is_ok_and(|s| s == self.binding.sealed_sha256)
    }
    pub(crate) fn matches_context(&self, context: &str) -> bool {
        self.valid() && self.binding.context_sha256 == context
    }
    pub(crate) fn wire(&self) -> CriterionWire {
        CriterionWire {
            revision: self.revision.clone(),
            calibration_sha256: self.calibration_sha256.clone(),
            limits: CriterionLimits {
                max_elapsed_ns: self.max_elapsed_ns,
                max_requested_heap_bytes: self.max_requested_heap_bytes,
                max_rss_bytes: self.max_rss_bytes,
                max_native_work: self.max_native_work,
            },
            binding: self.binding.clone(),
        }
    }
    pub(crate) fn restore(wire: CriterionWire, context: &str) -> Result<Self, ModValidationError> {
        let c = Self {
            revision: wire.revision,
            calibration_sha256: wire.calibration_sha256,
            max_elapsed_ns: wire.limits.max_elapsed_ns,
            max_requested_heap_bytes: wire.limits.max_requested_heap_bytes,
            max_rss_bytes: wire.limits.max_rss_bytes,
            max_native_work: wire.limits.max_native_work,
            binding: wire.binding,
        };
        if !c.matches_context(context) {
            return Err(ModValidationError::incomplete(
                "criterion.cache",
                "actual calibration or current build/profile/suite binding mismatch",
            ));
        }
        Ok(c)
    }
    /// This comparison is diagnostic, not an approval producer. The issuer only
    /// assesses metrics captured by its supervised processes.
    pub fn assess(&self, metrics: &LabMetrics) -> CriterionAssessment {
        if !self.valid() || !metrics.is_complete() {
            return CriterionAssessment::Incomplete {
                reason: "criterion or actual measurement completeness unavailable".into(),
            };
        }
        for sample in metrics.plain.iter().chain(&metrics.instrumented) {
            for (metric, actual, limit) in [
                ("elapsed_ns", sample.elapsed_ns, self.max_elapsed_ns),
                ("native_work", sample.native_work, self.max_native_work),
            ] {
                if actual > limit {
                    return CriterionAssessment::Exceeded {
                        metric: metric.into(),
                        actual,
                        limit,
                    };
                }
            }
        }
        for sample in &metrics.instrumented {
            for (metric, actual, limit) in [
                (
                    "requested_heap_bytes",
                    sample.requested_heap_peak_bytes.unwrap_or(0),
                    self.max_requested_heap_bytes,
                ),
                (
                    "rss_bytes",
                    sample
                        .rss_bytes
                        .unwrap_or(0)
                        .max(sample.rss_lifetime_peak_bytes.unwrap_or(0)),
                    self.max_rss_bytes,
                ),
            ] {
                if actual > limit {
                    return CriterionAssessment::Exceeded {
                        metric: metric.into(),
                        actual,
                        limit,
                    };
                }
            }
        }
        CriterionAssessment::Passed
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CriterionAssessment {
    Passed,
    Exceeded {
        metric: String,
        actual: u64,
        limit: u64,
    },
    Incomplete {
        reason: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabProfile {
    pub revision: String,
    pub repeats: u32,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub runtime_safety_ron: Vec<u8>,
    pub criterion: Option<LabCriterion>,
}
impl LabProfile {
    pub(crate) fn context(
        &self,
        suite: &LabSuite,
        pair: &ExecutionPairIdentity,
    ) -> Result<String, ModValidationError> {
        digest(&(
            suite,
            &self.revision,
            self.repeats,
            self.timeout_ms,
            self.max_output_bytes,
            &self.runtime_safety_ron,
            pair,
        ))
    }
    pub(crate) fn validate(&self, candidate: &Candidate) -> Result<(), ModValidationError> {
        if self.revision.is_empty()
            || self.revision.len() > 128
            || !(1..=31).contains(&self.repeats)
            || !(1..=300_000).contains(&self.timeout_ms)
            || !(1..=16 * 1024 * 1024).contains(&self.max_output_bytes)
            || self.runtime_safety_ron.len() > 64 * 1024
            || self.criterion.as_ref().is_some_and(|c| !c.valid())
        {
            return Err(ModValidationError::invalid(
                "lab_profile",
                "invalid finite trusted Lab profile",
            ));
        }
        let safety = crate::runtime_safety::RuntimeSafetySettings::from_ron(
            "registered Lab profile",
            &self.runtime_safety_ron,
        )
        .map_err(|e| ModValidationError::invalid("lab_profile.runtime_safety", e.to_string()))?;
        if safety.effective_sha256() != candidate.settings.effective_sha256()
            || safety.raw_sha256() != candidate.settings.raw_sha256()
        {
            return Err(ModValidationError::invalid(
                "lab_profile.runtime_safety",
                "candidate/profile mismatch",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabPhaseSample {
    pub phase: String,
    pub repeat_index: u32,
    pub elapsed_ns: u64,
    pub requested_heap_live_bytes: Option<u64>,
    pub requested_heap_peak_bytes: Option<u64>,
    pub rss_bytes: Option<u64>,
    /// Process-lifetime VmHWM, not a phase-local physical peak.
    pub rss_lifetime_peak_bytes: Option<u64>,
    pub native_work: u64,
    pub native_accounting: Option<serde_json::Value>,
    pub authoring_work: u64,
    pub sample_count: u64,
    pub dropped_samples: u64,
    pub missing_samples: u64,
    pub counter_overflowed: bool,
    pub window_uncertain: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabMetrics {
    pub plain: Vec<LabPhaseSample>,
    pub instrumented: Vec<LabPhaseSample>,
    pub pair: Option<ExecutionPairIdentity>,
    pub repeat_count: u32,
    pub complete: bool,
}
impl LabMetrics {
    pub fn is_complete(&self) -> bool {
        if !self.complete
            || !(1..=31).contains(&self.repeat_count)
            || self.pair.as_ref().is_none_or(|p| p.validate().is_err())
        {
            return false;
        }
        for (instrumented, rows) in [(false, &self.plain), (true, &self.instrumented)] {
            if rows.len() != self.repeat_count as usize * 4 {
                return false;
            }
            for (repeat, group) in rows.chunks_exact(4).enumerate() {
                for (sample, phase) in group
                    .iter()
                    .zip(["acquisition", "setup", "execute", "drop"])
                {
                    if sample.phase != phase
                        || sample.repeat_index != repeat as u32
                        || sample.sample_count == 0
                        || sample.dropped_samples != 0
                        || sample.missing_samples != 0
                        || sample.counter_overflowed
                        || sample.window_uncertain
                        || instrumented
                            && (sample.requested_heap_live_bytes.is_none()
                                || sample.requested_heap_peak_bytes.is_none()
                                || sample.rss_bytes.is_none()
                                || sample.rss_lifetime_peak_bytes.is_none()
                                || sample.native_accounting.as_ref().is_none_or(|v| {
                                    v.get("work").and_then(serde_json::Value::as_u64)
                                        != Some(sample.native_work)
                                        || v.get("clone_slots")
                                            .and_then(serde_json::Value::as_u64)
                                            .is_none()
                                        || v.get("admitted_bytes")
                                            .and_then(serde_json::Value::as_u64)
                                            .is_none()
                                        || v.get("sequence").and_then(serde_json::Value::as_u64)
                                            != Some(1)
                                }))
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabStatus {
    Passed,
    ContentFailed,
    Incomplete,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabOutcome {
    pub status: LabStatus,
    pub candidate_identity: String,
    pub suite_sha256: String,
    pub profile_sha256: String,
    pub coverage: CoverageReport,
    pub metrics: LabMetrics,
    pub reasons: Vec<String>,
    pub parity_sha256: String,
}
impl LabOutcome {
    pub fn is_passed(&self) -> bool {
        self.status == LabStatus::Passed
            && self.coverage.is_complete()
            && self.metrics.is_complete()
    }
}
pub struct LabRun {
    outcome: LabOutcome,
    evidence: Option<PathBuf>,
}
impl LabRun {
    pub fn evidence_directory(&self) -> Option<&Path> {
        self.evidence.as_deref()
    }
    pub fn calibrate(
        candidate: &Candidate,
        suite: &LabSuite,
        profile: &LabProfile,
        pair: &WorkerPair,
        work_root: &Path,
        cancel: &CancellationToken,
    ) -> Result<CalibrationRun, ModValidationError> {
        if profile.criterion.is_some() {
            return Err(ModValidationError::invalid(
                "calibration.profile",
                "use an unadopted diagnostic profile for calibration",
            ));
        }
        let context_sha256 = profile.context(suite, &pair.identity)?;
        let run = Self::supervised(candidate, suite, profile, pair, work_root, cancel)?;
        if !run.metrics().is_complete() {
            return Err(ModValidationError::incomplete(
                "calibration",
                "paired measurement incomplete; no adopted criterion",
            ));
        }
        Ok(CalibrationRun {
            run,
            context_sha256,
        })
    }
    pub fn outcome(&self) -> &LabOutcome {
        &self.outcome
    }
    pub fn coverage(&self) -> &CoverageReport {
        &self.outcome.coverage
    }
    pub fn metrics(&self) -> &LabMetrics {
        &self.outcome.metrics
    }
    /// A real single-process diagnostic. It can never issue an approval and is
    /// explicitly incomplete without supervised plain/instrumented metrics.
    pub fn diagnose(
        candidate: &Candidate,
        suite: &LabSuite,
        profile: &LabProfile,
        cancel: &CancellationToken,
    ) -> Result<Self, ModValidationError> {
        suite.validate()?;
        profile.validate(candidate)?;
        let report = run_driver(candidate, suite, cancel)?;
        Ok(Self{outcome:LabOutcome{status:LabStatus::Incomplete,candidate_identity:candidate.view.identity.clone(),suite_sha256:digest(suite)?,profile_sha256:digest(profile)?,coverage:report.coverage,metrics:LabMetrics::default(),reasons:vec!["diagnostic only: fresh paired process metrics and calibrated criterion required".into()],parity_sha256:digest(&report.cases)?},evidence:None})
    }
    pub(crate) fn supervised(
        candidate: &Candidate,
        suite: &LabSuite,
        profile: &LabProfile,
        pair: &WorkerPair,
        work_root: &Path,
        cancel: &CancellationToken,
    ) -> Result<Self, ModValidationError> {
        suite.validate()?;
        profile.validate(candidate)?;
        pair.identity.validate()?;
        BuildIdentity::check_current_core(&pair.identity)?;
        if profile.criterion.as_ref().is_some_and(|c| {
            !c.matches_context(&profile.context(suite, &pair.identity).unwrap_or_default())
        }) {
            return Err(ModValidationError::incomplete(
                "criterion.context",
                "adopted criterion belongs to a different actual build/profile/suite",
            ));
        }
        validate_private_dir(work_root)?;
        let mut directories = 0;
        for entry in std::fs::read_dir(work_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                return Err(ModValidationError::incomplete(
                    "worker.evidence",
                    "unexpected work-root member",
                ));
            }
            directories += 1;
            if directories >= 64 {
                return Err(ModValidationError::incomplete(
                    "worker.evidence",
                    "finite retained evidence directory ceiling",
                ));
            }
        }
        let run_id = Uuid::new_v4();
        let work = work_root.join(run_id.to_string());
        create_private_dir(&work)?;
        let execution = (|| {
            let source = work.join("content");
            create_private_dir(&source)?;
            candidate.frozen.materialize(&source)?;
            let request = WorkerRequest {
                schema_version: 1,
                nonce: run_id,
                candidate_identity: candidate.view.identity.clone(),
                root: source,
                registry: serde_json::to_vec(&candidate.registry)
                    .map_err(|e| ModValidationError::invalid("registry", e.to_string()))?,
                base_selection: candidate.base_selection.clone(),
                runtime_safety_ron: profile.runtime_safety_ron.clone(),
                suite: suite.clone(),
            };
            let request_path = work.join("request.json");
            std::fs::write(
                &request_path,
                serde_json::to_vec(&request)
                    .map_err(|e| ModValidationError::incomplete("worker.request", e.to_string()))?,
            )?;
            let mut metrics = LabMetrics {
                pair: Some(pair.identity.clone()),
                repeat_count: profile.repeats,
                ..Default::default()
            };
            let mut first: Option<DriverReport> = None;
            let mut failure: Option<ModValidationError> = None;
            for repeat in 0..profile.repeats {
                for (mode, binary) in [
                    (WorkerMode::Plain, &pair.plain),
                    (WorkerMode::Instrumented, &pair.instrumented),
                ] {
                    if cancel.is_cancelled() {
                        return Err(ModValidationError::new(
                            ModValidationErrorKind::Cancelled,
                            "worker",
                            "cancelled before process",
                        ));
                    }
                    let output = work.join(format!("{repeat}-{mode:?}.json"));
                    let bytes = spawn_bounded(
                        binary,
                        &[
                            "--cmva-lab-worker",
                            request_path.to_str().ok_or_else(|| {
                                ModValidationError::invalid("worker.path", "non UTF8")
                            })?,
                            mode.argument(),
                        ],
                        &output,
                        profile.timeout_ms,
                        profile.max_output_bytes,
                        cancel,
                    )?;
                    let response: WorkerResponse = serde_json::from_slice(&bytes).map_err(|e| {
                        ModValidationError::incomplete("worker.report", e.to_string())
                    })?;
                    let expected = if mode == WorkerMode::Plain {
                        &pair.identity.plain
                    } else {
                        &pair.identity.instrumented
                    };
                    let mut report = match response {
                        WorkerResponse::Failed(report) => {
                            if report.schema_version != 1
                                || report.nonce != run_id
                                || report.mode != mode
                                || &report.build != expected
                                || report.candidate_identity != candidate.view.identity
                                || report.error.field_path.len() > 4096
                                || report.error.reason.len() > 4096
                            {
                                return Err(ModValidationError::incomplete(
                                    "worker.binding",
                                    "failure nonce/build/mode/candidate mismatch",
                                ));
                            }
                            if first.is_some()
                                || failure.as_ref().is_some_and(|error| error != &report.error)
                            {
                                return Err(ModValidationError::incomplete(
                                    "worker.parity",
                                    "actual typed failure meaning differs between workers/repeats",
                                ));
                            }
                            failure = Some(report.error);
                            continue;
                        }
                        WorkerResponse::Complete(report) => {
                            if failure.is_some() {
                                return Err(ModValidationError::incomplete(
                                    "worker.parity",
                                    "one actual worker failed and another completed",
                                ));
                            }
                            report
                        }
                    };
                    if report.schema_version != 1
                        || report.nonce != run_id
                        || report.mode != mode
                        || &report.build != expected
                        || report.driver.candidate_identity != candidate.view.identity
                        || report.samples.is_empty()
                    {
                        return Err(ModValidationError::incomplete(
                            "worker.binding",
                            "nonce/build/mode/candidate/sample mismatch",
                        ));
                    }
                    if let Some(first) = &first {
                        if digest(&first.cases)? != digest(&report.driver.cases)?
                            || digest(&first.coverage)? != digest(&report.driver.coverage)?
                        {
                            return Err(ModValidationError::incomplete(
                                "worker.parity",
                                "command/error/gameplay/checkpoint/coverage parity mismatch",
                            ));
                        }
                    } else {
                        first = Some(report.driver.clone());
                    }
                    for sample in &mut report.samples {
                        sample.repeat_index = repeat;
                    }
                    if mode == WorkerMode::Plain {
                        metrics.plain.extend(report.samples);
                    } else {
                        metrics.instrumented.extend(report.samples);
                    }
                }
            }
            if let Some(error) = failure {
                // Both actual roles returned the same bounded error. No partial
                // measurement is promoted to a complete run or cached approval.
                return Err(error);
            }
            let first = first.ok_or_else(|| {
                ModValidationError::incomplete("worker.samples", "zero paired samples")
            })?;
            metrics.complete = true;
            metrics.complete = metrics.is_complete();
            let mut reasons = Vec::new();
            let mut status = LabStatus::Passed;
            if !metrics.complete {
                status = LabStatus::Incomplete;
                reasons.push("measurement window/sample/drop incomplete".into());
            }
            if !first.coverage.unsupported.is_empty() || first.coverage.overflow {
                status = LabStatus::Incomplete;
                reasons.push("unsupported/overflow coverage".into());
            } else if !first.coverage.is_complete() {
                status = LabStatus::Incomplete;
                reasons.push("required actual producer coverage not reached".into());
                for row in first
                    .coverage
                    .rows
                    .iter()
                    .filter(|row| row.counts[row.requirement.required_stage as usize] == 0)
                    .take(32)
                {
                    let key = &row.requirement.key;
                    reasons.push(format!(
                        "missing witness {:?} for {}:{} action={} effect={} producer={}",
                        row.requirement.required_stage,
                        key.definition_kind,
                        key.definition_id,
                        key.action_id,
                        key.effect_id,
                        key.producer_family
                    ));
                }
            }
            match &profile.criterion {
                None => {
                    status = LabStatus::Incomplete;
                    reasons.push("criterion not calibrated/adopted".into());
                }
                Some(c) => match c.assess(&metrics) {
                    CriterionAssessment::Passed => {}
                    CriterionAssessment::Incomplete { reason } => {
                        status = LabStatus::Incomplete;
                        reasons.push(reason);
                    }
                    CriterionAssessment::Exceeded {
                        metric,
                        actual,
                        limit,
                    } => {
                        if status != LabStatus::Incomplete {
                            status = LabStatus::ContentFailed;
                        }
                        reasons.push(format!("calibrated {metric} exceeded: {actual}>{limit}"));
                    }
                },
            }
            Ok(Self {
                outcome: LabOutcome {
                    status,
                    candidate_identity: candidate.view.identity.clone(),
                    suite_sha256: digest(suite)?,
                    profile_sha256: digest(profile)?,
                    coverage: first.coverage,
                    metrics,
                    reasons,
                    parity_sha256: digest(&first.cases)?,
                },
                evidence: Some(work.clone()),
            })
        })();
        // Every admitted run keeps nonce-bound requests and bounded reports,
        // including accepted runs. The finite work-directory admission above
        // prevents implicit unbounded evidence growth across restarts.
        execution
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabCaseReport {
    pub id: String,
    pub class: WorkloadClass,
    pub command_sha256: String,
    pub error_sha256: String,
    pub gameplay_sha256: String,
    pub checkpoint_sha256: String,
    pub accepted_steps: usize,
    pub native_work: u64,
    pub preparation: Option<authoring::LabPreparationReceipt>,
    #[serde(default)]
    pub removed_references: Vec<RemovedReferenceReport>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverReport {
    pub candidate_identity: String,
    pub cases: Vec<LabCaseReport>,
    pub coverage: CoverageReport,
}
pub fn run_driver(
    candidate: &Candidate,
    suite: &LabSuite,
    cancel: &CancellationToken,
) -> Result<DriverReport, ModValidationError> {
    suite.validate()?;
    let plan = CoveragePlan::for_suite(candidate, &suite.required_definitions)?;
    let mut coverage = plan.report.clone();
    let mut cases = Vec::new();
    for case in &suite.cases {
        let removed_references = prepare_removed_references(candidate, case, cancel)?;
        let scenario = TrialScenario::from_json_slice(&case.scenario, &ScenarioLimits::default())
            .map_err(|e| map_authoring("suite.scenario", e))?;
        let mut limits = TrialLimits::with_runtime_safety(&candidate.settings);
        limits.coverage = Some(Arc::clone(&plan));
        let mut trial = prepare_trial(
            candidate,
            scenario,
            case.preparation.clone(),
            limits,
            cancel,
        )
        .map_err(|e| map_authoring("lab.prepare", e.into_error()))?;
        let mut commands = Vec::new();
        let mut gameplay = Vec::new();
        let errors = Vec::<String>::new();
        while !trial.is_complete() {
            let step = trial
                .advance(cancel)
                .map_err(|e| map_authoring("lab.execute", e.into_error()))?;
            commands.push(step.public_result().cloned());
            gameplay.push(
                serde_json::to_value(
                    trial
                        .observation()
                        .map_err(|e| map_authoring("lab.observe", e))?,
                )
                .map_err(|e| ModValidationError::incomplete("lab.observe", e.to_string()))?,
            );
        }
        let report = trial
            .lab_coverage()
            .map_err(|e| map_authoring("lab.coverage", e))?;
        coverage
            .merge(&report)
            .map_err(|e| ModValidationError::incomplete("lab.coverage", format!("{e:?}")))?;
        record_removed_references(
            candidate,
            case,
            &trial,
            &removed_references,
            &mut coverage,
            cancel,
        )?;
        let checkpoint = trial
            .lab_checkpoint()
            .map_err(|e| map_authoring("lab.checkpoint", e))?;
        cases.push(LabCaseReport {
            id: case.id.clone(),
            class: case.class,
            command_sha256: digest(&commands)?,
            error_sha256: digest(&errors)?,
            gameplay_sha256: digest(&gameplay)?,
            checkpoint_sha256: digest(&checkpoint)?,
            accepted_steps: trial.next_step_index(),
            native_work: trial.accepted_work(),
            preparation: trial.lab_preparation().cloned(),
            removed_references,
        });
        drop(trial);
    }
    Ok(DriverReport {
        candidate_identity: candidate.view.identity.clone(),
        cases,
        coverage,
    })
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerMode {
    Plain,
    Instrumented,
}
impl WorkerMode {
    fn argument(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Instrumented => "instrumented",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerRequest {
    pub schema_version: u32,
    pub nonce: Uuid,
    pub candidate_identity: String,
    pub root: PathBuf,
    pub registry: Vec<u8>,
    pub base_selection: crate::game::data::content_pack::ContentSelection,
    pub runtime_safety_ron: Vec<u8>,
    pub suite: LabSuite,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerReport {
    pub schema_version: u32,
    pub nonce: Uuid,
    pub mode: WorkerMode,
    pub build: BuildIdentity,
    pub driver: DriverReport,
    pub samples: Vec<LabPhaseSample>,
}
/// A parsed diagnostic cannot issue authority. The supervisor checks every
/// binding and compares both actual workers before accepting an error meaning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "result", content = "report", rename_all = "snake_case")]
pub enum WorkerResponse {
    Complete(WorkerReport),
    Failed(WorkerFailure),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerFailure {
    pub schema_version: u32,
    pub nonce: Uuid,
    pub mode: WorkerMode,
    pub build: BuildIdentity,
    pub candidate_identity: String,
    pub error: ModValidationError,
}
/// OS/allocator collection belongs to the trusted host executable. Core owns
/// the exact phase and gameplay driver; a collector cannot issue approval.
pub trait WorkerMeasurement {
    fn begin(&mut self, phase: &'static str) -> Result<(), ModValidationError>;
    fn end(
        &mut self,
        phase: &'static str,
        elapsed_ns: u64,
        authoring_work: u64,
    ) -> Result<LabPhaseSample, ModValidationError>;
}
pub fn execute_worker(
    request: WorkerRequest,
    mode: WorkerMode,
    build: BuildIdentity,
    measurement: &mut impl WorkerMeasurement,
) -> Result<WorkerReport, ModValidationError> {
    if request.schema_version != 1 {
        return Err(ModValidationError::invalid(
            "worker.request.schema_version",
            "unsupported version",
        ));
    }
    request.suite.validate()?;
    let cancel = CancellationToken::default();
    let mut samples = Vec::with_capacity(4);
    measurement.begin("acquisition")?;
    let start = Instant::now();
    let settings = Arc::new(
        crate::runtime_safety::RuntimeSafetySettings::from_ron(
            "worker registered profile",
            &request.runtime_safety_ron,
        )
        .map_err(|e| ModValidationError::invalid("worker.runtime_safety", e.to_string()))?,
    );
    let limits = crate::game::data::content_pack::InputLimits::default();
    let base = super::TrustedBase::acquire(
        &request.root,
        request.base_selection.clone(),
        &limits,
        settings,
        &cancel,
    )?;
    let candidate = Candidate::acquire(&request.root, &request.registry, &base, &limits, &cancel)?;
    if candidate.view.identity != request.candidate_identity {
        return Err(ModValidationError::incomplete(
            "worker.candidate",
            "captured content differs from supervisor",
        ));
    }
    samples.push(measurement.end("acquisition", nanos(start), candidate.usage.work)?);
    measurement.begin("setup")?;
    let start = Instant::now();
    let plan = CoveragePlan::for_suite(&candidate, &request.suite.required_definitions)?;
    let mut coverage = plan.report.clone();
    let mut trials = Vec::with_capacity(request.suite.cases.len());
    let mut removed_reports = Vec::with_capacity(request.suite.cases.len());
    let mut setup_work = 0u64;
    for case in &request.suite.cases {
        let removed = prepare_removed_references(&candidate, case, &cancel)?;
        for report in &removed {
            setup_work = setup_work.checked_add(report.setup_work).ok_or_else(|| {
                ModValidationError::incomplete("removed.work", "counter overflow")
            })?;
        }
        removed_reports.push(removed);
        let scenario = TrialScenario::from_json_slice(&case.scenario, &ScenarioLimits::default())
            .map_err(|e| map_authoring("suite.scenario", e))?;
        let mut limits = TrialLimits::with_runtime_safety(&candidate.settings);
        limits.coverage = Some(Arc::clone(&plan));
        let trial = prepare_trial(
            &candidate,
            scenario,
            case.preparation.clone(),
            limits,
            &cancel,
        )
        .map_err(|e| map_authoring("lab.prepare", e.into_error()))?;
        setup_work = setup_work
            .checked_add(trial.accepted_work())
            .ok_or_else(|| ModValidationError::incomplete("lab.work", "counter overflow"))?;
        trials.push(trial);
    }
    samples.push(measurement.end("setup", nanos(start), setup_work)?);
    measurement.begin("execute")?;
    let start = Instant::now();
    let mut cases = Vec::with_capacity(trials.len());
    let mut execution_work = 0u64;
    for ((case, trial), removed_references) in request
        .suite
        .cases
        .iter()
        .zip(&mut trials)
        .zip(&removed_reports)
    {
        let before = trial.accepted_work();
        let mut commands = Vec::new();
        let mut gameplay = Vec::new();
        while !trial.is_complete() {
            let step = trial
                .advance(&cancel)
                .map_err(|e| map_authoring("lab.execute", e.into_error()))?;
            commands.push(step.public_result().cloned());
            gameplay.push(
                serde_json::to_value(
                    trial
                        .observation()
                        .map_err(|e| map_authoring("lab.observe", e))?,
                )
                .map_err(|e| ModValidationError::incomplete("lab.observe", e.to_string()))?,
            );
        }
        coverage
            .merge(
                &trial
                    .lab_coverage()
                    .map_err(|e| map_authoring("lab.coverage", e))?,
            )
            .map_err(|e| ModValidationError::incomplete("lab.coverage", format!("{e:?}")))?;
        let removed_work = record_removed_references(
            &candidate,
            case,
            trial,
            removed_references,
            &mut coverage,
            &cancel,
        )?;
        let checkpoint = trial
            .lab_checkpoint()
            .map_err(|e| map_authoring("lab.checkpoint", e))?;
        let work = trial
            .accepted_work()
            .checked_sub(before)
            .ok_or_else(|| ModValidationError::incomplete("lab.work", "counter underflow"))?;
        execution_work = execution_work
            .checked_add(work)
            .and_then(|n| n.checked_add(removed_work))
            .ok_or_else(|| ModValidationError::incomplete("lab.work", "counter overflow"))?;
        cases.push(LabCaseReport {
            id: case.id.clone(),
            class: case.class,
            command_sha256: digest(&commands)?,
            error_sha256: digest(&Vec::<String>::new())?,
            gameplay_sha256: digest(&gameplay)?,
            checkpoint_sha256: digest(&checkpoint)?,
            accepted_steps: trial.next_step_index(),
            native_work: 0,
            preparation: trial.lab_preparation().cloned(),
            removed_references: removed_references.clone(),
        });
    }
    samples.push(measurement.end("execute", nanos(start), execution_work)?);
    let identity = candidate.view.identity.clone();
    measurement.begin("drop")?;
    let start = Instant::now();
    drop(trials);
    drop(removed_reports);
    drop(plan);
    drop(candidate);
    drop(base);
    samples.push(measurement.end("drop", nanos(start), 0)?);
    Ok(WorkerReport {
        schema_version: 1,
        nonce: request.nonce,
        mode,
        build,
        driver: DriverReport {
            candidate_identity: identity,
            cases,
            coverage,
        },
        samples,
    })
}
fn nanos(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}
fn recipe_references(
    candidate: &Candidate,
    recipe: &authoring::LabPreparation,
    kind: crate::game::data::content_pack::DefinitionKind,
) -> std::collections::BTreeSet<String> {
    use crate::game::data::content_pack::DefinitionKind as K;
    let mut ids = std::collections::BTreeSet::new();
    if kind == K::PveEncounter {
        ids.insert(recipe.encounter_id.clone());
    }
    for group in recipe.employees.iter().filter(|g| g.count > 0) {
        match kind {
            K::Employee => {
                ids.insert(group.definition_id.clone());
            }
            K::Relic => ids.extend(group.relics.iter().map(|r| r.definition_id.clone())),
            K::Equipment => {
                if let Some(equipment) = &group.equipment_ids {
                    ids.extend(equipment.iter().cloned());
                } else if let Some(employee) = candidate
                    .generation()
                    .database()
                    .employee_definition_data
                    .get_by_id(&group.definition_id)
                {
                    ids.extend(employee.initial_loadout.equipment_ids.iter().cloned());
                }
            }
            K::SkillFragment => {
                if let Some(fragment) = &group.active_fragment_id {
                    ids.insert(fragment.clone());
                }
                if let Some(employee) = candidate
                    .generation()
                    .database()
                    .employee_definition_data
                    .get_by_id(&group.definition_id)
                {
                    ids.extend(
                        employee
                            .initial_loadout
                            .baseline_skill_fragment_ids
                            .iter()
                            .map(|id| id.as_str().to_owned()),
                    );
                    if group.active_fragment_id.is_none() {
                        ids.extend(
                            employee
                                .initial_loadout
                                .active_skill_fragment_id
                                .iter()
                                .map(|id| id.as_str().to_owned()),
                        );
                    }
                }
            }
            _ => {}
        }
    }
    ids
}
fn prepare_removed_references(
    candidate: &Candidate,
    case: &LabCase,
    cancel: &CancellationToken,
) -> Result<Vec<RemovedReferenceReport>, ModValidationError> {
    use crate::game::data::content_pack::DefinitionKind as K;
    let mut reports = Vec::with_capacity(case.removed_references.len());
    let mut keys = std::collections::BTreeSet::new();
    for reference in &case.removed_references {
        if !keys.insert(reference.key.clone())
            || !candidate.view.removed_definitions.contains(&reference.key)
            || candidate
                .publication
                .definitions
                .iter()
                .any(|d| d.key == reference.key)
            || !matches!(
                reference.key.kind,
                K::Employee | K::Equipment | K::SkillFragment | K::Relic | K::PveEncounter
            )
            || !recipe_references(candidate, &reference.preparation, reference.key.kind)
                .contains(&reference.key.id)
        {
            return Err(ModValidationError::incomplete(
                "removed.reference",
                "exact removed key and actual normal preparation reference required",
            ));
        }
        let base = candidate.base_authoring_content(cancel)?;
        let base_generation = base.token().identity().clone();
        let acquisition_work = base.validation_usage().work;
        let scenario =
            TrialScenario::from_json_slice(&reference.scenario, &ScenarioLimits::default())
                .map_err(|e| map_authoring("removed.scenario", e))?;
        let positive = TrialSession::prepare_with_lab(
            base,
            scenario,
            reference.preparation.clone(),
            TrialLimits::with_runtime_safety(&candidate.settings),
            cancel,
        )
        .map_err(|e| ModValidationError::incomplete("removed.base_preparation", e.to_string()))?;
        let base_preparation = positive
            .lab_preparation()
            .cloned()
            .filter(|r| r.deployed_count > 0 && !r.battle_id.is_nil())
            .ok_or_else(|| {
                ModValidationError::incomplete(
                    "removed.base_preparation",
                    "actual deployed positive base preparation required",
                )
            })?;
        let positive_work = positive.accepted_work();
        drop(positive);
        let scenario =
            TrialScenario::from_json_slice(&reference.scenario, &ScenarioLimits::default())
                .map_err(|e| map_authoring("removed.scenario", e))?;
        let failure = match TrialSession::prepare_with_lab(
            candidate.authoring_content(),
            scenario,
            reference.preparation.clone(),
            TrialLimits::with_runtime_safety(&candidate.settings),
            cancel,
        ) {
            Ok(_) => {
                return Err(ModValidationError::incomplete(
                    "removed.reference",
                    "removed reference unexpectedly prepared",
                ))
            }
            Err(error) => error,
        };
        if failure.position() != authoring::TrialPosition::PreparationReady {
            return Err(ModValidationError::incomplete(
                "removed.reference",
                "unrelated prefix/setup failure is not a removed reference witness",
            ));
        }
        let authoring::AuthoringError::Application { diagnostic, .. } = failure.error() else {
            return Err(ModValidationError::incomplete(
                "removed.reference",
                failure.to_string(),
            ));
        };
        let details = diagnostic.details().as_value();
        let exact = match reference.key.kind {
            K::Employee => {
                details["reason"]
                    == format!(
                        "lab preparation references missing employee '{}'",
                        reference.key.id
                    )
            }
            K::Equipment => reference.preparation.employees.iter().any(|employee| {
                details["reason"]
                    == format!(
                        "starter employee candidate '{}' references missing equipment '{}'",
                        employee.definition_id, reference.key.id
                    )
            }),
            K::SkillFragment => {
                details["reason"]
                    == format!(
                        "reward references missing skill fragment id '{}'",
                        reference.key.id
                    )
            }
            K::Relic => {
                details["reason"]
                    == format!(
                        "admin grant references missing relic '{}'",
                        reference.key.id
                    )
            }
            K::PveEncounter => {
                diagnostic.code() == "missing_resource" && details["resource"] == "PveEncounter"
            }
            _ => false,
        };
        if !exact {
            return Err(ModValidationError::incomplete(
                "removed.reference",
                "actual typed refusal differs from the exact removed normal reference",
            ));
        }
        let rejected_work = failure
            .attempt_usage()
            .ok_or_else(|| {
                ModValidationError::incomplete(
                    "removed.work",
                    "actual rejected preparation usage unavailable",
                )
            })?
            .work;
        let setup_work = acquisition_work
            .checked_add(positive_work)
            .and_then(|n| n.checked_add(rejected_work))
            .ok_or_else(|| ModValidationError::incomplete("removed.work", "counter overflow"))?;
        reports.push(RemovedReferenceReport {
            key: reference.key.clone(),
            base_generation,
            base_preparation,
            rejected_reference: diagnostic.clone(),
            setup_work,
        });
    }
    Ok(reports)
}
fn record_removed_references(
    candidate: &Candidate,
    case: &LabCase,
    trial: &TrialSession,
    proofs: &[RemovedReferenceReport],
    coverage: &mut CoverageReport,
    cancel: &CancellationToken,
) -> Result<u64, ModValidationError> {
    if proofs.is_empty() {
        return Ok(0);
    }
    let recipe = case.preparation.as_ref().ok_or_else(|| {
        ModValidationError::incomplete(
            "removed.surviving",
            "actual same-kind surviving preparation required",
        )
    })?;
    if !trial.is_complete()
        || !trial
            .lab_preparation()
            .is_some_and(|r| r.deployed_count > 0 && !r.battle_id.is_nil())
    {
        return Err(ModValidationError::incomplete(
            "removed.surviving",
            "actual completed deployed surviving trial required",
        ));
    }
    let (result, usage) = crate::authoring::control::run(
        &candidate.settings.authoring_execution(),
        cancel,
        crate::authoring::ExecutionPhase::TrialExecution,
        || {
            for proof in proofs {
                crate::authoring::control::charge(
                    candidate.publication.definitions.len() as u64 + 1,
                    "lab_removed_surviving",
                )?;
                if !recipe_references(candidate, recipe, proof.key.kind)
                    .iter()
                    .any(|id| {
                        id != &proof.key.id
                            && candidate
                                .publication
                                .definitions
                                .iter()
                                .any(|d| d.key.kind == proof.key.kind && &d.key.id == id)
                    })
                {
                    return Err(crate::game::behavior::GameError::InvalidAction);
                }
                let kind = format!("{:?}", proof.key.kind);
                for (family, stage) in [
                    ("removed_absence", super::CoverageStage::CatalogValidated),
                    ("removed_reference", super::CoverageStage::Ineligible),
                    ("removed_surviving", super::CoverageStage::ConsumedOrEffect),
                ] {
                    coverage.record_definition(&kind, &proof.key.id, family, stage)?;
                }
            }
            Ok::<_, crate::game::behavior::GameError>(())
        },
    )
    .map_err(|e| ModValidationError::execution("removed.coverage", e))?;
    result.map_err(|e| ModValidationError::incomplete("removed.surviving", format!("{e:?}")))?;
    Ok(usage.work)
}
fn prepare_trial(
    candidate: &Candidate,
    scenario: TrialScenario,
    recipe: Option<authoring::LabPreparation>,
    limits: TrialLimits,
    cancel: &CancellationToken,
) -> Result<TrialSession, authoring::TrialFailure> {
    match recipe {
        Some(recipe) => TrialSession::prepare_with_lab(
            candidate.authoring_content(),
            scenario,
            recipe,
            limits,
            cancel,
        ),
        None => TrialSession::prepare(candidate.authoring_content(), scenario, limits, cancel),
    }
}
#[derive(Clone)]
pub struct WorkerPair {
    pub(crate) plain: PathBuf,
    pub(crate) instrumented: PathBuf,
    pub(crate) identity: ExecutionPairIdentity,
    evidence: Vec<PathBuf>,
}
impl WorkerPair {
    pub fn open(
        plain: PathBuf,
        instrumented: PathBuf,
        environment: String,
    ) -> Result<Self, ModValidationError> {
        let mut builds = Vec::new();
        let mut evidence = Vec::new();
        let cancel = CancellationToken::default();
        for binary in [&plain, &instrumented] {
            let path = std::env::temp_dir().join(format!("cmva-build-{}.json", Uuid::new_v4()));
            let result = spawn_bounded(
                binary,
                &["--cmva-build-identity"],
                &path,
                10_000,
                65536,
                &cancel,
            );
            evidence.push(path);
            let identity: BuildIdentity = serde_json::from_slice(&result?)
                .map_err(|e| ModValidationError::incomplete("worker.build", e.to_string()))?;
            if identity.executable_sha256 != executable_digest(binary)? {
                return Err(ModValidationError::incomplete(
                    "worker.build",
                    "binary changed/false digest",
                ));
            }
            builds.push(identity);
        }
        let identity = ExecutionPairIdentity {
            plain: builds.remove(0),
            instrumented: builds.remove(0),
            environment,
            measurement_method: "cmva-process-ledger-v1".into(),
        };
        identity.validate()?;
        Ok(Self {
            plain,
            instrumented,
            identity,
            evidence,
        })
    }
    /// Bounded stdout, stderr and process sidecars for the actual identity commands.
    pub fn evidence_paths(&self) -> &[PathBuf] {
        &self.evidence
    }
    pub fn identity(&self) -> &ExecutionPairIdentity {
        &self.identity
    }
}
fn spawn_bounded(
    binary: &Path,
    args: &[&str],
    output: &Path,
    timeout_ms: u64,
    cap: u64,
    cancel: &CancellationToken,
) -> Result<Vec<u8>, ModValidationError> {
    let expected = executable_digest(binary)?;
    let child = std::process::Command::new(binary)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let mut owned = OwnedChild { child };
    let stdout = owned
        .child
        .stdout
        .take()
        .ok_or_else(|| ModValidationError::incomplete("worker.stdout", "pipe missing"))?;
    let stderr = owned
        .child
        .stderr
        .take()
        .ok_or_else(|| ModValidationError::incomplete("worker.stderr", "pipe missing"))?;
    let total = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let exceeded = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stderr_path = output.with_extension("stderr");
    let mut readers = Vec::new();
    for (stream, path) in [
        (
            Box::new(stdout) as Box<dyn std::io::Read + Send>,
            output.to_owned(),
        ),
        (
            Box::new(stderr) as Box<dyn std::io::Read + Send>,
            stderr_path.clone(),
        ),
    ] {
        let total = Arc::clone(&total);
        let exceeded = Arc::clone(&exceeded);
        match std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(move || read_process_stream(stream, &path, cap, total, exceeded))
        {
            Ok(reader) => readers.push(reader),
            Err(e) => {
                owned.reap();
                for reader in readers {
                    let _ = reader.join();
                }
                return Err(e.into());
            }
        }
    }
    let start = Instant::now();
    let execution = (|| loop {
        if cancel.is_cancelled()
            || start.elapsed() > Duration::from_millis(timeout_ms)
            || exceeded.load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(ModValidationError::new(
                if cancel.is_cancelled() {
                    ModValidationErrorKind::Cancelled
                } else {
                    ModValidationErrorKind::Incomplete
                },
                "worker.supervisor",
                "cancel/timeout/combined stream ceiling; child reaped",
            ));
        }
        if let Some(status) = owned.child.try_wait()? {
            return Ok(status);
        }
        std::thread::sleep(Duration::from_millis(10));
    })();
    owned.reap();
    let mut stream_error = None;
    for reader in readers {
        match reader.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => stream_error = Some(e),
            Err(_) => stream_error = Some(std::io::Error::other("stream collector panicked")),
        }
    }
    let final_digest = executable_digest(binary);
    let evidence = serde_json::json!({"binary_sha256_before":expected,"binary_sha256_after":final_digest.as_ref().ok(),"args":args,"elapsed_ns":nanos(start),"exit_code":execution.as_ref().ok().and_then(|s|s.code()),"success":execution.as_ref().is_ok_and(|s|s.success()),"supervisor_error":execution.as_ref().err(),"stream_error":stream_error.as_ref().map(ToString::to_string),"stream_bytes":total.load(std::sync::atomic::Ordering::SeqCst),"stream_ceiling":cap,"stream_overflow":exceeded.load(std::sync::atomic::Ordering::SeqCst),"stdout":output,"stderr":stderr_path,"child_reaped":true});
    std::fs::write(
        output.with_extension("process.json"),
        serde_json::to_vec(&evidence)
            .map_err(|e| ModValidationError::incomplete("worker.evidence", e.to_string()))?,
    )?;
    let status = execution?;
    if let Some(e) = stream_error {
        return Err(e.into());
    }
    if !status.success()
        || final_digest? != expected
        || exceeded.load(std::sync::atomic::Ordering::SeqCst)
    {
        return Err(ModValidationError::incomplete(
            "worker.exit",
            format!(
                "worker failure/binary drift/output ceiling; bounded raw evidence {}",
                output.display()
            ),
        ));
    }
    Ok(std::fs::read(output)?)
}
struct OwnedChild {
    child: std::process::Child,
}
impl OwnedChild {
    fn reap(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.reap();
    }
}
fn read_process_stream(
    mut stream: Box<dyn std::io::Read + Send>,
    path: &Path,
    cap: u64,
    total: Arc<std::sync::atomic::AtomicU64>,
    exceeded: Arc<std::sync::atomic::AtomicBool>,
) -> std::io::Result<()> {
    use std::sync::atomic::Ordering;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    let mut bytes = [0; 4096];
    loop {
        let n = stream.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        let mut previous = total.load(Ordering::SeqCst);
        let admitted = loop {
            let allowed = cap.saturating_sub(previous).min(n as u64);
            match total.compare_exchange(
                previous,
                previous + allowed,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break allowed as usize,
                Err(next) => previous = next,
            }
        };
        file.write_all(&bytes[..admitted])?;
        if admitted < n {
            exceeded.store(true, Ordering::SeqCst);
            break;
        }
    }
    file.flush()?;
    Ok(())
}
fn executable_digest(path: &Path) -> Result<String, ModValidationError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(ModValidationError::invalid(
            "worker.binary",
            "regular binary required",
        ));
    }
    let mut h = Sha256::new();
    let mut bytes = [0; 65536];
    loop {
        let n = file.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        h.update(&bytes[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub(crate) fn create_private_dir(path: &Path) -> Result<(), ModValidationError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        let mut b = std::fs::DirBuilder::new();
        b.mode(0o700);
        b.create(path)?;
        validate_private_dir(path)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(ModValidationError::incomplete(
            "worker.platform",
            "private worker root unsupported",
        ))
    }
}
fn validate_private_dir(path: &Path) -> Result<(), ModValidationError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = std::fs::symlink_metadata(path)?;
        if !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o777 != 0o700
        {
            return Err(ModValidationError::incomplete(
                "worker.root",
                "worker root must be an owned private directory without a symlink",
            ));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(ModValidationError::incomplete(
            "worker.platform",
            "private worker root unsupported",
        ))
    }
}
fn hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn map_authoring(field: &str, e: authoring::AuthoringError) -> ModValidationError {
    if field == "lab.prepare" && !matches!(e, authoring::AuthoringError::Execution { .. }) {
        return ModValidationError::incomplete(field, e.to_string());
    }
    match e {
        authoring::AuthoringError::Execution { diagnostic } => {
            ModValidationError::execution(field, diagnostic)
        }
        e => ModValidationError::new(ModValidationErrorKind::ContentFailed, field, e.to_string()),
    }
}
