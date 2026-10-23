use super::{
    cache::{AcceptedRecord, CacheRoot, Publication},
    digest,
    lab::{create_private_dir, WorkerPair},
    Candidate, LabOutcome, LabProfile, LabRun, LabSuite, ModValidationError,
    ModValidationErrorKind, RequiredConfiguration,
};
use crate::{authoring::CancellationToken, game::data::ValidatedGameData};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use uuid::Uuid;
const MAX_JOBS: usize = 64;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct JobId(Uuid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Pending,
    Running,
    Passed,
    ContentFailed,
    Incomplete,
    Cancelled,
    DependencyHeld,
    Stale,
    RecoveryPending,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelDisposition {
    Cancelled,
    TooLatePublicationAccepted,
}
#[derive(Debug, Clone, Serialize)]
pub struct JobView {
    pub id: JobId,
    pub state: JobState,
    pub selection_revision: u64,
    pub current: bool,
    pub candidate_identity: String,
    pub cache_hit: bool,
    pub error: Option<ModValidationError>,
    pub outcome: Option<LabOutcome>,
}
struct Authority {
    issuer: Uuid,
}
#[derive(Clone)]
pub struct ApprovedGeneration {
    authority: Arc<Authority>,
    revision: u64,
    candidate: Arc<Candidate>,
    result_sha256: String,
}
impl std::fmt::Debug for ApprovedGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprovedGeneration")
            .field("issuer", &self.authority.issuer)
            .field("revision", &self.revision)
            .field("candidate", &self.candidate.view.identity)
            .finish()
    }
}
impl ApprovedGeneration {
    pub fn generation(&self) -> &ValidatedGameData {
        self.candidate.generation()
    }
    pub fn required_configuration(&self) -> &RequiredConfiguration {
        self.candidate.required_configuration()
    }
    pub fn candidate_identity(&self) -> &str {
        &self.candidate.view.identity
    }
    pub fn selection_revision(&self) -> u64 {
        self.revision
    }
    pub fn result_sha256(&self) -> &str {
        &self.result_sha256
    }
}
struct Job {
    view: JobView,
    candidate: Arc<Candidate>,
    cancel: CancellationToken,
    accepted: Option<AcceptedRecord>,
    approval: Option<ApprovedGeneration>,
    suite: LabSuite,
    profile: LabProfile,
}
struct State {
    revision: u64,
    jobs: BTreeMap<JobId, Job>,
}
pub struct ValidationIssuer {
    authority: Arc<Authority>,
    cache: CacheRoot,
    work_root: PathBuf,
    suite: LabSuite,
    profile: LabProfile,
    pair: WorkerPair,
    state: Mutex<State>,
}
impl ValidationIssuer {
    pub fn matches_execution_context(
        &self,
        profile: &LabProfile,
        pair: &WorkerPair,
    ) -> Result<bool, ModValidationError> {
        let mut own = self.profile.clone();
        let mut requested = profile.clone();
        own.criterion = None;
        requested.criterion = None;
        Ok(digest(&own)? == digest(&requested)? && self.pair.identity == pair.identity)
    }
    pub fn open(
        cache_root: &Path,
        suite: LabSuite,
        mut profile: LabProfile,
        pair: WorkerPair,
    ) -> Result<Arc<Self>, ModValidationError> {
        suite.validate()?;
        pair.identity.validate()?;
        super::BuildIdentity::check_current_core(&pair.identity)?;
        let cache = CacheRoot::open(cache_root)?;
        let criterion_context = profile.context(&suite, &pair.identity)?;
        if let Some(criterion) = &profile.criterion {
            cache.adopt_criterion(&criterion_context, criterion)?;
        } else {
            profile.criterion = cache.criterion(&criterion_context)?;
        }
        let work_root = cache_root.canonicalize()?.join("workers");
        if !work_root.exists() {
            create_private_dir(&work_root)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let m = std::fs::symlink_metadata(&work_root)?;
            if !m.is_dir()
                || m.mode() & 0o077 != 0
                || m.uid() != rustix::process::geteuid().as_raw()
            {
                return Err(ModValidationError::incomplete(
                    "workers.root",
                    "owned private directory required",
                ));
            }
        }
        Ok(Arc::new(Self {
            authority: Arc::new(Authority {
                issuer: cache.issuer(),
            }),
            cache,
            work_root,
            suite,
            profile,
            pair,
            state: Mutex::new(State {
                revision: 0,
                jobs: BTreeMap::new(),
            }),
        }))
    }
    fn state(&self) -> Result<std::sync::MutexGuard<'_, State>, ModValidationError> {
        self.state.lock().map_err(|_| {
            ModValidationError::incomplete("issuer.state", "poisoned single-writer owner")
        })
    }
    pub fn begin(&self, candidate: Candidate) -> Result<JobId, ModValidationError> {
        self.begin_for_suite(candidate, self.suite.clone())
    }
    /// A trusted owner may choose a bounded individual suite. Only the exact
    /// privately authenticated calibration context is reusable for that job.
    pub fn begin_for_suite(
        &self,
        candidate: Candidate,
        suite: LabSuite,
    ) -> Result<JobId, ModValidationError> {
        suite.validate()?;
        let mut profile = self.profile.clone();
        let context = profile.context(&suite, &self.pair.identity)?;
        if !profile
            .criterion
            .as_ref()
            .is_some_and(|c| c.matches_context(&context))
        {
            profile.criterion = self.cache.criterion(&context)?;
        }
        profile.validate(&candidate)?;
        let mut state = self.state()?;
        if state.jobs.len() >= MAX_JOBS {
            return Err(ModValidationError::incomplete(
                "issuer.jobs",
                "finite job admission exhausted",
            ));
        }
        state.revision = state.revision.checked_add(1).ok_or_else(|| {
            ModValidationError::incomplete("issuer.revision", "revision exhausted")
        })?;
        let revision = state.revision;
        for job in state.jobs.values_mut() {
            if matches!(
                job.view.state,
                JobState::Pending | JobState::Running | JobState::RecoveryPending
            ) {
                job.view.state = JobState::Stale;
                job.view.current = false;
                job.cancel.cancel();
            }
        }
        let id = JobId(Uuid::new_v4());
        let view = JobView {
            id,
            state: JobState::Pending,
            selection_revision: revision,
            current: true,
            candidate_identity: candidate.view.identity.clone(),
            cache_hit: false,
            error: None,
            outcome: None,
        };
        state.jobs.insert(
            id,
            Job {
                view,
                candidate: Arc::new(candidate),
                cancel: CancellationToken::default(),
                accepted: None,
                approval: None,
                suite,
                profile,
            },
        );
        Ok(id)
    }
    pub fn query(&self, id: JobId) -> Result<JobView, ModValidationError> {
        let state = self.state()?;
        let job = state
            .jobs
            .get(&id)
            .ok_or_else(|| ModValidationError::invalid("job", "unknown job"))?;
        let mut view = job.view.clone();
        view.current = view.selection_revision == state.revision;
        Ok(view)
    }
    pub fn cancel(&self, id: JobId) -> Result<CancelDisposition, ModValidationError> {
        let mut state = self.state()?;
        let job = state
            .jobs
            .get_mut(&id)
            .ok_or_else(|| ModValidationError::invalid("job", "unknown job"))?;
        if job.accepted.is_some() {
            return Ok(CancelDisposition::TooLatePublicationAccepted);
        }
        job.cancel.cancel();
        job.view.state = JobState::Cancelled;
        job.approval = None;
        Ok(CancelDisposition::Cancelled)
    }
    pub fn invalidate_selection(&self) -> Result<u64, ModValidationError> {
        let mut state = self.state()?;
        state.revision = state.revision.checked_add(1).ok_or_else(|| {
            ModValidationError::incomplete("issuer.revision", "revision exhausted")
        })?;
        for job in state.jobs.values_mut() {
            if matches!(
                job.view.state,
                JobState::Pending | JobState::Running | JobState::RecoveryPending
            ) {
                job.view.state = JobState::Stale;
                job.cancel.cancel();
            }
        }
        Ok(state.revision)
    }
    fn key(
        &self,
        candidate: &Candidate,
        suite: &LabSuite,
        profile: &LabProfile,
    ) -> Result<String, ModValidationError> {
        digest(&(
            1_u32,
            self.authority.issuer,
            &candidate.view,
            suite,
            profile,
            &self.pair.identity,
        ))
    }
    pub fn run(&self, id: JobId) -> Result<JobView, ModValidationError> {
        let (candidate, cancel, revision, suite, profile) = {
            let mut state = self.state()?;
            let current = state.revision;
            let job = state
                .jobs
                .get_mut(&id)
                .ok_or_else(|| ModValidationError::invalid("job", "unknown job"))?;
            if job.view.state != JobState::Pending
                || job.view.selection_revision != current
                || job.cancel.is_cancelled()
            {
                return Err(ModValidationError::new(
                    ModValidationErrorKind::Stale,
                    "job",
                    "job is not current pending",
                ));
            }
            job.view.state = JobState::Running;
            (
                Arc::clone(&job.candidate),
                job.cancel.clone(),
                current,
                job.suite.clone(),
                job.profile.clone(),
            )
        };
        let key = self.key(&candidate, &suite, &profile)?;
        let cached = self.cache.find(&key);
        let result = match cached {
            Ok(Some(record)) => {
                if record.result.candidate_identity != candidate.view.identity
                    || record.result.suite_sha256 != digest(&suite)?
                    || record.result.profile_sha256 != digest(&profile)?
                    || record.result.metrics.pair.as_ref() != Some(&self.pair.identity)
                {
                    Err(ModValidationError::incomplete(
                        "cache.binding",
                        "complete exact candidate/profile/suite/pair mismatch",
                    ))
                } else {
                    self.cache.recover(&record).map(|()| (record.result, true))
                }
            }
            Ok(None) => LabRun::supervised(
                &candidate,
                &suite,
                &profile,
                &self.pair,
                &self.work_root,
                &cancel,
            )
            .map(|run| (run.outcome().clone(), false)),
            Err(e) => Err(e),
        };
        #[cfg(test)]
        super::u11_cache_hooks::after_lab(
            id,
            result.as_ref().ok().map(|(outcome, hit)| (outcome, *hit)),
        );
        let mut state = self.state()?;
        let current = state.revision;
        let job = state
            .jobs
            .get_mut(&id)
            .ok_or_else(|| ModValidationError::invalid("job", "job disappeared"))?;
        if current != revision {
            job.view.state = JobState::Stale;
            return Ok(job.view.clone());
        }
        if job.cancel.is_cancelled() {
            job.view.state = JobState::Cancelled;
            return Ok(job.view.clone());
        }
        let (outcome, hit) = match result {
            Ok(value) => value,
            Err(e) => {
                job.view.state = state_from_error(e.kind);
                job.view.error = Some(e);
                return Ok(job.view.clone());
            }
        };
        job.view.cache_hit = hit;
        job.view.outcome = Some(outcome.clone());
        if !outcome.is_passed() {
            job.view.state = match outcome.status {
                super::lab::LabStatus::ContentFailed => JobState::ContentFailed,
                super::lab::LabStatus::Cancelled => JobState::Cancelled,
                _ => JobState::Incomplete,
            };
            return Ok(job.view.clone());
        }
        let record = AcceptedRecord {
            schema_version: 1,
            issuer: self.authority.issuer,
            key,
            revision,
            nonce: id.0,
            result: outcome,
        };
        let publication = if hit {
            Ok(Publication::Durable)
        } else {
            self.cache.publish(&record, || {
                if job.cancel.is_cancelled() {
                    Err(ModValidationError::new(
                        ModValidationErrorKind::Cancelled,
                        "cache.accept",
                        "cancelled before rename",
                    ))
                } else {
                    Ok(())
                }
            })
        };
        match publication {
            Ok(publication) => {
                job.accepted = Some(record);
                match publication {
                    Publication::Durable => issue(&self.authority, job)?,
                    Publication::RecoveryPending(reason) => {
                        job.view.state = JobState::RecoveryPending;
                        job.view.error =
                            Some(ModValidationError::incomplete("cache.durability", reason));
                    }
                }
            }
            Err(e) => {
                job.view.state = state_from_error(e.kind);
                job.view.error = Some(e);
            }
        }
        Ok(job.view.clone())
    }
    pub fn recover(&self, id: JobId) -> Result<JobView, ModValidationError> {
        let mut state = self.state()?;
        let current = state.revision;
        let job = state
            .jobs
            .get_mut(&id)
            .ok_or_else(|| ModValidationError::invalid("job", "unknown job"))?;
        if job.view.state != JobState::RecoveryPending || job.view.selection_revision != current {
            return Err(ModValidationError::new(
                ModValidationErrorKind::Stale,
                "cache.recovery",
                "recovery is not current",
            ));
        }
        let record = job.accepted.as_ref().ok_or_else(|| {
            ModValidationError::incomplete("cache.recovery", "no accepted complete record")
        })?;
        if record.key != self.key(&job.candidate, &job.suite, &job.profile)? {
            return Err(ModValidationError::incomplete(
                "cache.recovery",
                "exact identity changed",
            ));
        }
        self.cache.recover(record)?;
        issue(&self.authority, job)?;
        Ok(job.view.clone())
    }
    pub fn approval(&self, id: JobId) -> Result<ApprovedGeneration, ModValidationError> {
        let state = self.state()?;
        let job = state
            .jobs
            .get(&id)
            .ok_or_else(|| ModValidationError::invalid("job", "unknown job"))?;
        if job.view.state != JobState::Passed || job.view.selection_revision != state.revision {
            return Err(ModValidationError::new(
                ModValidationErrorKind::Stale,
                "approval",
                "no current completed approval",
            ));
        }
        job.approval
            .clone()
            .ok_or_else(|| ModValidationError::incomplete("approval", "capability unavailable"))
    }
    pub(crate) fn check_approval(
        &self,
        approved: &ApprovedGeneration,
    ) -> Result<(), ModValidationError> {
        let state = self.state()?;
        if !Arc::ptr_eq(&self.authority, &approved.authority) {
            return Err(ModValidationError::new(
                ModValidationErrorKind::ForeignIssuer,
                "apply",
                "foreign capability owner",
            ));
        }
        if state.revision != approved.revision {
            return Err(ModValidationError::new(
                ModValidationErrorKind::Stale,
                "apply",
                "selection revision changed",
            ));
        }
        Ok(())
    }
    pub(crate) fn with_current_approval<T>(
        &self,
        approved: &ApprovedGeneration,
        commit: impl FnOnce() -> Result<T, ModValidationError>,
    ) -> Result<T, ModValidationError> {
        let state = self.state()?;
        if !Arc::ptr_eq(&self.authority, &approved.authority) {
            return Err(ModValidationError::new(
                ModValidationErrorKind::ForeignIssuer,
                "apply",
                "foreign capability owner",
            ));
        }
        if state.revision != approved.revision {
            return Err(ModValidationError::new(
                ModValidationErrorKind::Stale,
                "apply",
                "selection revision changed",
            ));
        }
        commit()
    }
}
fn issue(authority: &Arc<Authority>, job: &mut Job) -> Result<(), ModValidationError> {
    let record = job
        .accepted
        .as_ref()
        .ok_or_else(|| ModValidationError::incomplete("approval", "accepted record missing"))?;
    job.approval = Some(ApprovedGeneration {
        authority: Arc::clone(authority),
        revision: job.view.selection_revision,
        candidate: Arc::clone(&job.candidate),
        result_sha256: digest(&record.result)?,
    });
    job.view.state = JobState::Passed;
    job.view.error = None;
    Ok(())
}
fn state_from_error(kind: ModValidationErrorKind) -> JobState {
    match kind {
        ModValidationErrorKind::ContentFailed | ModValidationErrorKind::InvalidInput => {
            JobState::ContentFailed
        }
        ModValidationErrorKind::Cancelled => JobState::Cancelled,
        ModValidationErrorKind::Stale => JobState::Stale,
        ModValidationErrorKind::DependencyHeld => JobState::DependencyHeld,
        _ => JobState::Incomplete,
    }
}
