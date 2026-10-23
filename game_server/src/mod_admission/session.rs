//! Trusted local/server admission configuration and the normal issuer owner.
use game_core::{
    authoring::CancellationToken,
    game::data::content_pack::{ContentSelection, InputLimits},
    mod_validation::{
        ApprovedHost, BuildIdentity, Candidate, CandidateView, CriterionLimits, HostSelectionView,
        JobId, JobState, JobView, LabProfile, LabRun, LabSuite, ModRegistry, ModValidationError,
        ModValidationErrorKind, TrustedBase, ValidationIssuer, WorkerPair,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionConfig {
    pub schema_version: u32,
    pub content_root: PathBuf,
    pub registry_path: PathBuf,
    pub base_selection_path: PathBuf,
    pub runtime_safety_path: PathBuf,
    pub suite_path: PathBuf,
    #[serde(default)]
    pub individual_suite_paths: BTreeMap<String, PathBuf>,
    pub issuer_root: PathBuf,
    pub plain_worker: PathBuf,
    pub instrumented_worker: PathBuf,
    pub environment: String,
    pub profile_revision: String,
    pub repeats: u32,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}
impl AdmissionConfig {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 64 * 1024 {
            return Err("finite admission configuration byte ceiling".into());
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if value.schema_version != 1 {
            return Err("unsupported admission configuration version".into());
        }
        Ok(value)
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        Self::from_json_slice(&read(path, 64 * 1024)?)
    }
}
struct Inputs {
    base: TrustedBase,
    registry: ModRegistry,
    suite: LabSuite,
    individual_suites: BTreeMap<String, LabSuite>,
    profile: LabProfile,
    pair: WorkerPair,
}
impl AdmissionConfig {
    fn inputs(&self) -> Result<Inputs, String> {
        if self.schema_version != 1 {
            return Err("unsupported admission configuration version".into());
        }
        let safety = read(&self.runtime_safety_path, 64 * 1024)?;
        let settings = Arc::new(
            game_core::runtime_safety::RuntimeSafetySettings::from_ron(
                "trusted admission profile",
                &safety,
            )
            .map_err(|e| e.to_string())?,
        );
        let base_selection: ContentSelection =
            serde_json::from_slice(&read(&self.base_selection_path, 64 * 1024)?)
                .map_err(|e| e.to_string())?;
        let cancel = CancellationToken::default();
        let base = TrustedBase::acquire(
            &self.content_root,
            base_selection,
            &InputLimits::default(),
            settings,
            &cancel,
        )
        .map_err(|e| e.to_string())?;
        let registry = ModRegistry::from_json_slice(&read(&self.registry_path, 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        if self.individual_suite_paths.len() > 128
            || self
                .individual_suite_paths
                .keys()
                .any(|id| !registry.mods.iter().any(|m| &m.mod_id == id))
        {
            return Err("finite exact registered individual suite references required".into());
        }
        let suite: LabSuite = serde_json::from_slice(&read(&self.suite_path, 16 * 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        let individual_suites = self
            .individual_suite_paths
            .iter()
            .map(|(id, path)| {
                let suite = serde_json::from_slice(&read(path, 16 * 1024 * 1024)?)
                    .map_err(|e| e.to_string())?;
                Ok((id.clone(), suite))
            })
            .collect::<Result<_, String>>()?;
        let profile = LabProfile {
            revision: self.profile_revision.clone(),
            repeats: self.repeats,
            timeout_ms: self.timeout_ms,
            max_output_bytes: self.max_output_bytes,
            runtime_safety_ron: safety,
            criterion: None,
        };
        let host = BuildIdentity::current(&std::env::current_exe().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if host.profile != "release" || !host.features.is_empty() {
            return Err("normal admission requires the actual default-release host".into());
        }
        let environment = format!(
            "{};normal_host={}",
            self.environment,
            serde_json::to_string(&host).map_err(|e| e.to_string())?
        );
        let pair = WorkerPair::open(
            self.plain_worker.clone(),
            self.instrumented_worker.clone(),
            environment,
        )
        .map_err(|e| e.to_string())?;
        if pair.identity().plain.source_compiler_sha256 != host.source_compiler_sha256
            || pair.identity().plain.dependency_graph_sha256 != host.dependency_graph_sha256
            || pair.identity().plain.target != host.target
            || pair.identity().plain.profile != host.profile
        {
            return Err(
                "normal host and actual validation pair differ in source/compiler/target/profile"
                    .into(),
            );
        }
        Ok(Inputs {
            base,
            registry,
            suite,
            individual_suites,
            profile,
            pair,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct IndividualValidation {
    pub mod_id: String,
    pub state: JobState,
    pub candidate: Option<CandidateView>,
    pub job: Option<JobView>,
    pub error: Option<ModValidationError>,
}
#[derive(Debug, Clone, Serialize)]
pub struct AdmissionReport {
    pub individuals: Vec<IndividualValidation>,
    pub held_mods: Vec<String>,
    pub selected_mods: Vec<String>,
    pub combination: Option<JobView>,
    pub combination_error: Option<ModValidationError>,
}
pub struct AdmissionSession {
    pub issuer: Arc<ValidationIssuer>,
    pub host: Arc<ApprovedHost>,
    pub job: JobView,
    pub report: AdmissionReport,
    individual_suites: BTreeMap<String, LabSuite>,
}
fn ordered_mods(registry: &ModRegistry) -> Vec<String> {
    fn visit(
        id: &str,
        registry: &ModRegistry,
        visited: &mut BTreeSet<String>,
        result: &mut Vec<String>,
    ) {
        if !visited.insert(id.to_owned()) {
            return;
        }
        let m = registry
            .mods
            .iter()
            .find(|m| m.mod_id == id)
            .expect("validated registry closure");
        for dependency in &m.dependencies {
            visit(dependency, registry, visited, result);
        }
        result.push(id.to_owned());
    }
    let mut result = Vec::new();
    let mut visited = BTreeSet::new();
    for id in &registry.selection.mods {
        visit(id, registry, &mut visited, &mut result);
    }
    result
}
impl AdmissionSession {
    fn admit(
        config: &AdmissionConfig,
    ) -> Result<
        (
            AdmissionReport,
            Option<(Arc<ValidationIssuer>, JobId)>,
            BTreeMap<String, LabSuite>,
        ),
        String,
    > {
        Self::admit_with_owner(config, None)
    }
    fn admit_with_owner(
        config: &AdmissionConfig,
        existing: Option<&Arc<ValidationIssuer>>,
    ) -> Result<
        (
            AdmissionReport,
            Option<(Arc<ValidationIssuer>, JobId)>,
            BTreeMap<String, LabSuite>,
        ),
        String,
    > {
        let input = config.inputs()?;
        if let Some(issuer) = existing {
            if !issuer
                .matches_execution_context(&input.profile, &input.pair)
                .map_err(|e| e.to_string())?
            {
                return Err(
                    "update requires the installed issuer's exact actual execution/profile context"
                        .into(),
                );
            }
        }
        let mut report = AdmissionReport {
            individuals: Vec::new(),
            held_mods: Vec::new(),
            selected_mods: Vec::new(),
            combination: None,
            combination_error: None,
        };
        let cancel = CancellationToken::default();
        for id in ordered_mods(&input.registry) {
            let definition = input.registry.mods.iter().find(|m| m.mod_id == id).unwrap();
            if definition
                .dependencies
                .iter()
                .any(|d| report.held_mods.contains(d))
            {
                report.held_mods.push(id.clone());
                report.individuals.push(IndividualValidation {
                    mod_id: id,
                    state: JobState::DependencyHeld,
                    candidate: None,
                    job: None,
                    error: Some(ModValidationError::new(
                        ModValidationErrorKind::DependencyHeld,
                        "mods.dependencies",
                        "a required mod is held",
                    )),
                });
                continue;
            }
            let mut registry = input.registry.clone();
            registry.selection.mods = vec![id.clone()];
            let acquired = Candidate::acquire_individual(
                &config.content_root,
                &serde_json::to_vec(&registry).map_err(|e| e.to_string())?,
                &id,
                &input.base,
                &InputLimits::default(),
                &cancel,
            );
            let candidate = match acquired {
                Ok(candidate) => candidate,
                Err(error) => {
                    report.held_mods.push(id.clone());
                    report.individuals.push(IndividualValidation {
                        mod_id: id,
                        state: if matches!(
                            error.kind,
                            ModValidationErrorKind::ContentFailed
                                | ModValidationErrorKind::InvalidInput
                        ) {
                            JobState::ContentFailed
                        } else {
                            JobState::Incomplete
                        },
                        candidate: None,
                        job: None,
                        error: Some(error),
                    });
                    continue;
                }
            };
            let view = candidate.view().clone();
            let suite = input
                .individual_suites
                .get(&id)
                .cloned()
                .unwrap_or_else(|| input.suite.clone());
            // Each closure runs the same real issuer supervisor. Sequential owner
            // scopes release the private lock; authenticated independent results
            // remain available even when the final combination later fails.
            let result = (|| -> Result<JobView, ModValidationError> {
                let issuer = match existing {
                    Some(issuer) => Arc::clone(issuer),
                    None => ValidationIssuer::open(
                        &config.issuer_root,
                        suite.clone(),
                        input.profile.clone(),
                        input.pair.clone(),
                    )?,
                };
                let job = issuer.begin_for_suite(candidate, suite)?;
                issuer.run(job)
            })();
            match result {
                Ok(job) => {
                    if job.state != JobState::Passed {
                        report.held_mods.push(id.clone());
                    }
                    report.individuals.push(IndividualValidation {
                        mod_id: id,
                        state: job.state,
                        candidate: Some(view),
                        error: job.error.clone(),
                        job: Some(job),
                    });
                }
                Err(error) => {
                    report.held_mods.push(id.clone());
                    report.individuals.push(IndividualValidation {
                        mod_id: id,
                        state: JobState::Incomplete,
                        candidate: Some(view),
                        job: None,
                        error: Some(error),
                    });
                }
            }
        }
        let mut registry = input.registry.clone();
        registry
            .selection
            .mods
            .retain(|id| !report.held_mods.contains(id));
        report.selected_mods = registry.selection.mods.clone();
        if !input.registry.selection.mods.is_empty() && registry.selection.mods.is_empty() {
            report.combination_error = Some(ModValidationError::new(
                ModValidationErrorKind::DependencyHeld,
                "selection.mods",
                "all requested mods are held; no replacement selection was applied",
            ));
            return Ok((report, None, input.individual_suites));
        }
        let candidate = match Candidate::acquire(
            &config.content_root,
            &serde_json::to_vec(&registry).map_err(|e| e.to_string())?,
            &input.base,
            &InputLimits::default(),
            &cancel,
        ) {
            Ok(candidate) => candidate,
            Err(error) => {
                report.combination_error = Some(error);
                return Ok((report, None, input.individual_suites));
            }
        };
        for individual in report
            .individuals
            .iter()
            .filter(|r| r.state == JobState::Passed)
        {
            if let Some(view) = &individual.candidate {
                if view
                    .source_receipt
                    .files
                    .iter()
                    .any(|file| !candidate.view().source_receipt.files.contains(file))
                {
                    report.combination_error = Some(ModValidationError::new(
                        ModValidationErrorKind::Stale,
                        "selection.sources",
                        "individual captured sources changed before final combination acquisition",
                    ));
                    return Ok((report, None, input.individual_suites));
                }
            }
        }
        let issuer = match existing {
            Some(issuer) => Arc::clone(issuer),
            None => ValidationIssuer::open(
                &config.issuer_root,
                input.suite.clone(),
                input.profile,
                input.pair,
            )
            .map_err(|e| e.to_string())?,
        };
        let id = issuer
            .begin_for_suite(candidate, input.suite)
            .map_err(|e| e.to_string())?;
        report.combination = Some(issuer.run(id).map_err(|e| e.to_string())?);
        Ok((report, Some((issuer, id)), input.individual_suites))
    }
    pub fn validate(config: &AdmissionConfig) -> Result<AdmissionReport, String> {
        Self::admit(config).map(|(report, _, _)| report)
    }
    pub fn open(config: &AdmissionConfig) -> Result<Self, String> {
        let (report, owner, individual_suites) = Self::admit(config)?;
        let (issuer, id) = owner.ok_or_else(|| {
            format!(
                "normal admission held: {}",
                serde_json::to_string(&report).unwrap_or_default()
            )
        })?;
        let approval = issuer.approval(id).map_err(|e| {
            format!(
                "normal admission refused: {e}; {}",
                serde_json::to_string(&report).unwrap_or_default()
            )
        })?;
        let host = ApprovedHost::new(Arc::clone(&issuer), approval).map_err(|e| e.to_string())?;
        let job = report.combination.clone().expect("actual combination job");
        Ok(Self {
            issuer,
            host,
            job,
            report,
            individual_suites,
        })
    }
    pub fn query(&self) -> Result<HostSelectionView, String> {
        self.host.query().map_err(|e| e.to_string())
    }
    /// Fresh registry acquisition uses the same independent/dependency/final
    /// orchestration as startup, including independently valid mods when one
    /// selected pack cannot be acquired. Failed validation preserves the host.
    pub fn validate_configuration_and_apply(
        &mut self,
        config: &AdmissionConfig,
    ) -> Result<AdmissionReport, String> {
        let (report, owner, _) = Self::admit_with_owner(config, Some(&self.issuer))?;
        if let Some((issuer, id)) = owner {
            if let Ok(approval) = issuer.approval(id) {
                self.host.apply(approval).map_err(|e| e.to_string())?;
            }
        }
        if let Some(job) = &report.combination {
            self.job = job.clone();
        }
        self.report = report.clone();
        Ok(report)
    }
    /// Frozen-candidate updates use this session's original admitted suite
    /// contexts. Fresh configurations use validate_configuration_and_apply.
    pub fn validate_and_apply_report(
        &mut self,
        candidate: Candidate,
    ) -> Result<AdmissionReport, String> {
        let mut report = AdmissionReport {
            individuals: Vec::new(),
            held_mods: Vec::new(),
            selected_mods: Vec::new(),
            combination: None,
            combination_error: None,
        };
        let cancel = CancellationToken::default();
        for id in ordered_mods(candidate.registry()) {
            let definition = candidate
                .registry()
                .mods
                .iter()
                .find(|m| m.mod_id == id)
                .expect("validated registry");
            if definition
                .dependencies
                .iter()
                .any(|d| report.held_mods.contains(d))
            {
                report.held_mods.push(id.clone());
                report.individuals.push(IndividualValidation {
                    mod_id: id,
                    state: JobState::DependencyHeld,
                    candidate: None,
                    job: None,
                    error: Some(ModValidationError::new(
                        ModValidationErrorKind::DependencyHeld,
                        "mods.dependencies",
                        "a required mod is held",
                    )),
                });
                continue;
            }
            let result = candidate
                .individual_from_capture(&id, &cancel)
                .and_then(|individual| {
                    let view = individual.view().clone();
                    let job_id = match self.individual_suites.get(&id) {
                        Some(suite) => self.issuer.begin_for_suite(individual, suite.clone()),
                        None => self.issuer.begin(individual),
                    }?;
                    self.issuer.run(job_id).map(|job| (view, job))
                });
            match result {
                Ok((view, job)) => {
                    if job.state != JobState::Passed {
                        report.held_mods.push(id.clone());
                    }
                    report.individuals.push(IndividualValidation {
                        mod_id: id,
                        state: job.state,
                        candidate: Some(view),
                        error: job.error.clone(),
                        job: Some(job),
                    });
                }
                Err(error) => {
                    report.held_mods.push(id.clone());
                    let state = if matches!(
                        error.kind,
                        ModValidationErrorKind::ContentFailed
                            | ModValidationErrorKind::InvalidInput
                    ) {
                        JobState::ContentFailed
                    } else {
                        JobState::Incomplete
                    };
                    report.individuals.push(IndividualValidation {
                        mod_id: id,
                        state,
                        candidate: None,
                        job: None,
                        error: Some(error),
                    });
                }
            }
        }
        report.selected_mods = candidate
            .registry()
            .selection
            .mods
            .iter()
            .filter(|id| !report.held_mods.contains(id))
            .cloned()
            .collect();
        if !candidate.registry().selection.mods.is_empty() && report.selected_mods.is_empty() {
            report.combination_error = Some(ModValidationError::new(
                ModValidationErrorKind::DependencyHeld,
                "selection.mods",
                "all requested mods are held; no replacement selection was applied",
            ));
        } else {
            match candidate.selected_from_capture(report.selected_mods.clone(), &cancel) {
                Err(error) => report.combination_error = Some(error),
                Ok(final_candidate) => {
                    let id = self
                        .issuer
                        .begin(final_candidate)
                        .map_err(|e| e.to_string())?;
                    let job = self.issuer.run(id).map_err(|e| e.to_string())?;
                    if let Ok(approval) = self.issuer.approval(id) {
                        self.host.apply(approval).map_err(|e| e.to_string())?;
                    }
                    self.job = job.clone();
                    report.combination = Some(job);
                }
            }
        }
        self.report = report.clone();
        Ok(report)
    }
    pub fn validate_and_apply(&mut self, candidate: Candidate) -> Result<JobView, String> {
        let report = self.validate_and_apply_report(candidate)?;
        report.combination.ok_or_else(|| {
            report
                .combination_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "no actual final combination job admitted".into())
        })
    }
    /// Invoked only by a trusted host owner after Root has reviewed actual raw
    /// calibration. The private issuer authenticates the adopted criterion.
    pub fn calibrate_and_adopt(
        config: &AdmissionConfig,
        evidence_root: &Path,
        revision: String,
        limits: CriterionLimits,
    ) -> Result<serde_json::Value, String> {
        let mut input = config.inputs()?;
        let candidate = Candidate::acquire(
            &config.content_root,
            &serde_json::to_vec(&input.registry).map_err(|e| e.to_string())?,
            &input.base,
            &InputLimits::default(),
            &CancellationToken::default(),
        )
        .map_err(|e| e.to_string())?;
        let run = LabRun::calibrate(
            &candidate,
            &input.suite,
            &input.profile,
            &input.pair,
            evidence_root,
            &CancellationToken::default(),
        )
        .map_err(|e| e.to_string())?;
        let criterion = run.adopt(revision, limits).map_err(|e| e.to_string())?;
        let result = serde_json::json!({"criterion": criterion, "outcome": run.outcome(), "evidence": run.evidence_directory()});
        input.profile.criterion = Some(criterion);
        // A private issuer constructor persists the actual calibration binding;
        // arbitrary input DTOs cannot deserialize this proof.
        let _issuer =
            ValidationIssuer::open(&config.issuer_root, input.suite, input.profile, input.pair)
                .map_err(|e| e.to_string())?;
        Ok(result)
    }
    pub fn calibrate(
        config: &AdmissionConfig,
        evidence_root: &Path,
    ) -> Result<serde_json::Value, String> {
        let input = config.inputs()?;
        let candidate = Candidate::acquire(
            &config.content_root,
            &serde_json::to_vec(&input.registry).map_err(|e| e.to_string())?,
            &input.base,
            &InputLimits::default(),
            &CancellationToken::default(),
        )
        .map_err(|e| e.to_string())?;
        let run = LabRun::calibrate(
            &candidate,
            &input.suite,
            &input.profile,
            &input.pair,
            evidence_root,
            &CancellationToken::default(),
        )
        .map_err(|e| e.to_string())?;
        Ok(
            serde_json::json!({"candidate":candidate.view(), "pair":input.pair.identity(), "outcome":run.outcome(), "evidence":run.evidence_directory(), "criterion_adopted":false}),
        )
    }
}
/// Local administration runs in the real host executable so the calibration
/// context includes the executable that will later own normal admission.
pub fn administration(args: &[String]) -> Result<Option<serde_json::Value>, String> {
    let Some(command) = args.first() else {
        return Ok(None);
    };
    if command == "--cmva-coverage-support" {
        if args.len() != 1 {
            return Err("coverage support query takes no arguments".into());
        }
        return serde_json::to_value(game_core::mod_validation::coverage_support())
            .map(Some)
            .map_err(|e| e.to_string());
    }
    if !matches!(
        command.as_str(),
        "--cmva-calibrate" | "--cmva-adopt-calibration"
    ) {
        return Ok(None);
    }
    let expected = if command == "--cmva-calibrate" { 3 } else { 5 };
    if args.len() != expected {
        return Err("calibration requires admission.json, existing empty evidence directory, and adoption additionally requires revision and reviewed limits.json".into());
    }
    let config = AdmissionConfig::read(Path::new(&args[1]))?;
    let root = Path::new(&args[2]);
    let metadata = std::fs::symlink_metadata(root).map_err(|e| e.to_string())?;
    if !metadata.is_dir()
        || std::fs::read_dir(root)
            .map_err(|e| e.to_string())?
            .next()
            .is_some()
    {
        return Err("fresh empty calibration evidence directory required".into());
    }
    let result = if command == "--cmva-calibrate" {
        AdmissionSession::calibrate(&config, root)?
    } else {
        let limits: CriterionLimits =
            serde_json::from_slice(&read(Path::new(&args[4]), 64 * 1024)?)
                .map_err(|e| e.to_string())?;
        AdmissionSession::calibrate_and_adopt(&config, root, args[3].clone(), limits)?
    };
    Ok(Some(result))
}
pub(crate) fn read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > limit {
        return Err(format!(
            "{}: regular bounded trusted input required",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("trusted input byte ceiling".into());
    }
    Ok(bytes)
}
