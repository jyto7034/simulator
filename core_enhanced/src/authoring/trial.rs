//! Opaque prepared trials over the real application and observation ports.

use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use super::{
    clone_shape::{controlled_clone, CloneShape},
    control::{self, CloneMeter, NativeStorageProvenance, Resource},
    output::{self, PreparedReceipt},
    resolve, AuthoringContent, AuthoringError, CancellationToken, ExecutionLimits, ExecutionPhase,
    ExecutionUsage, RequestFailure, TrialClock, TrialScenario, TrialStep,
};
use crate::application::{ApplicationSnapshotKind, CoreApplication};

/// Trusted host settings, validated before construction or execution.
#[derive(Debug, Clone)]
pub struct TrialLimits {
    pub execution: ExecutionLimits,
    pub max_total_work: u64,
    pub max_receipt_bytes: usize,
    pub trace: super::trace::TraceOptions,
    pub amount_observation: bool,
}

impl Default for TrialLimits {
    fn default() -> Self {
        Self {
            execution: ExecutionLimits::default(),
            max_total_work: crate::runtime_safety::RuntimeSafetySettings::embedded()
                .trial_max_total_work(),
            max_receipt_bytes: crate::runtime_safety::RuntimeSafetySettings::embedded()
                .trial_max_receipt_bytes(),
            trace: Default::default(),
            amount_observation: false,
        }
    }
}

impl TrialLimits {
    pub fn with_runtime_safety(settings: &crate::runtime_safety::RuntimeSafetySettings) -> Self {
        Self {
            execution: settings.authoring_execution(),
            max_total_work: settings.trial_max_total_work(),
            max_receipt_bytes: settings.trial_max_receipt_bytes(),
            trace: Default::default(),
            amount_observation: false,
        }
    }

    fn remaining(
        &self,
        used: u64,
        phase: ExecutionPhase,
    ) -> Result<ExecutionLimits, AuthoringError> {
        // Validate the original settings before clipping; an invalid large
        // caller limit must not become valid merely because little fuel remains.
        self.execution.validate(phase).map_err(execution_error)?;
        if self.max_total_work == 0
            || self.max_total_work > 1 << 38
            || self.max_receipt_bytes == 0
            || self.max_receipt_bytes > 64 * 1024 * 1024
            || !self.trace.is_valid()
        {
            return Err(request_error(
                phase,
                "authoring_trial_limits_invalid",
                "Trusted trial limits exceed their finite ceilings.",
            ));
        }
        if used >= self.max_total_work {
            return Err(request_error(
                phase,
                "authoring_trial_work_limit",
                "The trial has no remaining work allowance.",
            ));
        }
        let mut limits = self.execution.clone();
        limits.max_work = limits.max_work.min(self.max_total_work - used);
        Ok(limits)
    }
}

pub(super) struct AcceptedStep {
    pub public_result: Option<Value>,
    pub receipt: PreparedReceipt,
}

pub(super) struct TrialState {
    application: CoreApplication,
    native_storage: NativeStorageProvenance,
    content: Arc<AuthoringContent>,
    scenario: Arc<TrialScenario>,
    limits: TrialLimits,
    initial_receipt: Arc<PreparedReceipt>,
    preparation_receipt: Option<Arc<PreparedReceipt>>,
    records: Vec<Arc<AcceptedStep>>,
    copied_bytes: usize,
    receipt_bytes: usize,
    accepted_work: u64,
}

#[derive(Serialize)]
struct ResolvedInput {
    command: Value,
    context: Value,
}

/// Position in the original finite input; preparation and execution share indexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "position", rename_all = "snake_case")]
pub enum TrialPosition {
    Initial,
    Step { index: usize },
    PreparationReady,
    End,
}

/// The original diagnostic remains intact, including a more precise Receipt phase.
#[derive(Debug, Serialize)]
pub struct TrialFailure {
    position: TrialPosition,
    operation_phase: ExecutionPhase,
    error: AuthoringError,
}
impl TrialFailure {
    fn new(
        position: TrialPosition,
        operation_phase: ExecutionPhase,
        error: AuthoringError,
    ) -> Self {
        Self {
            position,
            operation_phase,
            error,
        }
    }
    pub fn position(&self) -> TrialPosition {
        self.position
    }
    pub fn operation_phase(&self) -> ExecutionPhase {
        self.operation_phase
    }
    pub fn error(&self) -> &AuthoringError {
        &self.error
    }
    pub fn into_error(self) -> AuthoringError {
        self.error
    }
}
impl std::fmt::Display for TrialFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.position, self.error)
    }
}
impl std::error::Error for TrialFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReceiptView<'a> {
    document: &'a str,
    total_bytes: usize,
}
impl<'a> ReceiptView<'a> {
    pub fn document(self) -> &'a str {
        self.document
    }
    /// Aggregate mandatory bytes through this receipt, not just its own length.
    pub fn total_bytes(self) -> usize {
        self.total_bytes
    }
}
impl<'a> From<&'a PreparedReceipt> for ReceiptView<'a> {
    fn from(receipt: &'a PreparedReceipt) -> Self {
        Self {
            document: &receipt.document,
            total_bytes: receipt.total_bytes,
        }
    }
}

#[derive(Clone, Copy)]
pub struct StepView<'a> {
    index: usize,
    step: &'a AcceptedStep,
}
impl<'a> StepView<'a> {
    pub fn index(self) -> usize {
        self.index
    }
    pub fn receipt(self) -> ReceiptView<'a> {
        (&self.step.receipt).into()
    }
    /// None is an accepted empty slot (for example an assertion), not JSON null.
    pub fn public_result(self) -> Option<&'a Value> {
        self.step.public_result.as_ref()
    }
}
impl std::fmt::Debug for StepView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StepView")
            .field("index", &self.index)
            .field("receipt_bytes", &self.step.receipt.document.len())
            .finish_non_exhaustive()
    }
}

/// An actual prepared in-memory trial. No public clone or mutable inner authority.
pub struct TrialSession {
    state: TrialState,
}
impl std::fmt::Debug for TrialSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrialSession")
            .field("next_step_index", &self.next_step_index())
            .field("is_complete", &self.is_complete())
            .finish_non_exhaustive()
    }
}
impl TrialSession {
    pub fn prepare(
        content: AuthoringContent,
        scenario: TrialScenario,
        limits: TrialLimits,
        cancellation: &CancellationToken,
    ) -> Result<Self, TrialFailure> {
        TrialState::prepare_positioned(content, scenario, limits, cancellation)
            .map(|state| Self { state })
    }

    pub fn replace(
        &mut self,
        content: AuthoringContent,
        scenario: TrialScenario,
        limits: TrialLimits,
        cancellation: &CancellationToken,
    ) -> Result<ReceiptView<'_>, TrialFailure> {
        let candidate = Self::prepare(content, scenario, limits, cancellation)?;
        *self = candidate;
        Ok(self.ready_receipt())
    }

    pub fn advance(
        &mut self,
        cancellation: &CancellationToken,
    ) -> Result<StepView<'_>, TrialFailure> {
        let index = self.next_step_index();
        let phase = ExecutionPhase::TrialExecution;
        let position = if self.is_complete() {
            TrialPosition::End
        } else {
            TrialPosition::Step { index }
        };
        self.state
            .advance(cancellation)
            .map(|step| StepView { index, step })
            .map_err(|error| TrialFailure::new(position, phase, error))
    }

    /// Executes only remaining explicit input. It does not advance until victory.
    pub fn run_to_end(&mut self, cancellation: &CancellationToken) -> Result<(), TrialFailure> {
        while !self.is_complete() {
            self.advance(cancellation)?;
        }
        Ok(())
    }
    pub fn next_step_index(&self) -> usize {
        self.state.records.len()
    }
    pub fn is_complete(&self) -> bool {
        self.state.scenario.step(self.next_step_index()).is_none()
    }
    pub fn initial_receipt(&self) -> ReceiptView<'_> {
        self.state.initial_receipt.as_ref().into()
    }
    pub fn ready_receipt(&self) -> ReceiptView<'_> {
        self.state
            .preparation_receipt
            .as_deref()
            .expect("public session is prepared")
            .into()
    }
    pub fn step(&self, index: usize) -> Option<StepView<'_>> {
        self.state
            .records
            .get(index)
            .map(|step| StepView { index, step })
    }
    pub fn steps(&self) -> impl ExactSizeIterator<Item = StepView<'_>> {
        self.state
            .records
            .iter()
            .enumerate()
            .map(|(index, step)| StepView { index, step })
    }
    /// Existing encoded documents in acceptance order, without collecting or encoding.
    pub fn receipt_documents(&self) -> impl Iterator<Item = &str> {
        let (preparation, execution) = self
            .state
            .records
            .split_at(self.state.scenario.preparation_steps().len());
        std::iter::once(self.initial_receipt().document())
            .chain(
                preparation
                    .iter()
                    .map(|step| step.receipt.document.as_str()),
            )
            .chain(std::iter::once(self.ready_receipt().document()))
            .chain(execution.iter().map(|step| step.receipt.document.as_str()))
    }
    pub fn content_identity(&self) -> &crate::game::data::content_pack::ContentIdentity {
        self.state.application.content_identity()
    }
    pub fn runtime_safety(&self) -> &crate::runtime_safety::RuntimeSafetySettings {
        self.state.application.runtime_safety()
    }
    pub fn initial(&self) -> &super::TrialInitial {
        self.state.scenario.initial()
    }
    pub fn input_sha256(&self) -> &str {
        self.state.scenario.input_sha256()
    }
    pub fn limits(&self) -> &TrialLimits {
        &self.state.limits
    }
    pub fn accepted_work(&self) -> u64 {
        self.state.accepted_work
    }
    pub fn copied_bytes(&self) -> usize {
        self.state.copied_bytes
    }
    pub fn receipt_bytes(&self) -> usize {
        self.state.receipt_bytes
    }

    pub fn lifecycle(&self) -> super::LifecycleView {
        super::view::LifecycleView(
            self.state
                .application
                .authoring_lifecycle()
                .expect("public authoring lifecycle"),
        )
    }
    pub fn observation(&self) -> Result<super::ObservationView<'_>, AuthoringError> {
        self.state
            .application
            .authoring_observation()
            .map(super::view::ObservationView)
            .map_err(|diagnostic| AuthoringError::Application {
                phase: ExecutionPhase::Inspection,
                diagnostic,
            })
    }
    pub fn trace(&self) -> Result<super::SessionTraceView<'_>, AuthoringError> {
        self.state
            .application
            .authoring_trace()
            .map(super::view::SessionTraceView)
            .map_err(|diagnostic| AuthoringError::Application {
                phase: ExecutionPhase::Inspection,
                diagnostic,
            })
    }

    /// A separately bounded live query; its work never consumes accepted trial quota.
    pub fn snapshot(
        &self,
        kind: ApplicationSnapshotKind,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        self.inspect(limits, cancellation, |application| {
            application.snapshot(kind)
        })
    }

    pub fn reconnect(
        &self,
        after_seq: u64,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        self.inspect(limits, cancellation, |application| {
            application.active_battle_reconnect(after_seq)
        })
    }

    /// Execute a closed Preview command on a controlled candidate and discard
    /// it. Context and tactical focus retain the existing application semantics.
    pub fn preview(
        &self,
        request: super::TrialPreview,
        context: crate::application::ApplicationExecutionContext,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        self.inspect(limits, cancellation, |application| {
            let command =
                crate::application::ApplicationCommand::for_authoring_preview(request, &context)?;
            let mut candidate = application.fork_for_authoring()?;
            candidate.execute(self.initial().player_id, command, context)
        })
    }

    fn inspect<T: Serialize>(
        &self,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
        query: impl FnOnce(&CoreApplication) -> Result<T, crate::application::ApplicationError>,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        let phase = ExecutionPhase::Inspection;
        let (document, usage, _) = control::run_with_provenance(
            limits,
            cancellation,
            phase,
            self.state.native_storage,
            || {
                let result = query(&self.state.application)
                    .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?;
                output::encode_result(&result)
            },
        )
        .map_err(execution_error)?;
        Ok(super::AuthoringQueryResult::prepared(document?, usage))
    }

    pub fn describe(
        &self,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        super::describe(&self.state.content, limits, cancellation)
    }
    pub fn explain(
        &self,
        query: &super::ExplainQuery,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        super::explain(&self.state.content, query, limits, cancellation)
    }

    pub fn preview_amount(
        &self,
        query: super::AmountQuery,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        self.inspect_amount(query, false, limits, cancellation)
    }

    pub fn explain_amount(
        &self,
        query: super::AmountQuery,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        self.inspect_amount(query, true, limits, cancellation)
    }

    fn inspect_amount(
        &self,
        query: super::AmountQuery,
        explanation: bool,
        limits: &ExecutionLimits,
        cancellation: &CancellationToken,
    ) -> Result<super::AuthoringQueryResult, AuthoringError> {
        let phase = ExecutionPhase::Inspection;
        let (document, usage, _) = control::run_with_provenance(
            limits,
            cancellation,
            phase,
            self.state.native_storage,
            || super::amount_query::encode(&self.state.application, query, explanation),
        )
        .map_err(execution_error)?;
        Ok(super::AuthoringQueryResult::prepared(document?, usage))
    }
}

/// Failure owns the last accepted session, if preparation succeeded. Provisional
/// preparation records are never represented as accepted steps.
#[derive(Debug)]
pub struct SimulationFailure {
    accepted: Option<TrialSession>,
    failure: TrialFailure,
}
impl SimulationFailure {
    pub fn accepted(&self) -> Option<&TrialSession> {
        self.accepted.as_ref()
    }
    pub fn failure(&self) -> &TrialFailure {
        &self.failure
    }
    pub fn into_parts(self) -> (Option<TrialSession>, TrialFailure) {
        (self.accepted, self.failure)
    }
}
impl std::fmt::Display for SimulationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.failure.fmt(f)
    }
}
impl std::error::Error for SimulationFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.failure)
    }
}

pub fn simulate(
    content: AuthoringContent,
    scenario: TrialScenario,
    limits: TrialLimits,
    cancellation: &CancellationToken,
) -> Result<TrialSession, SimulationFailure> {
    let mut session =
        TrialSession::prepare(content, scenario, limits, cancellation).map_err(|failure| {
            SimulationFailure {
                accepted: None,
                failure,
            }
        })?;
    match session.run_to_end(cancellation) {
        Ok(()) => Ok(session),
        Err(failure) => Err(SimulationFailure {
            accepted: Some(session),
            failure,
        }),
    }
}

impl TrialState {
    #[cfg(test)]
    pub(super) fn prepare_owned(
        content: AuthoringContent,
        scenario: TrialScenario,
        limits: TrialLimits,
        cancellation: &CancellationToken,
    ) -> Result<Self, AuthoringError> {
        Self::prepare_positioned(content, scenario, limits, cancellation)
            .map_err(|failure| failure.error)
    }

    fn prepare_positioned(
        content: AuthoringContent,
        scenario: TrialScenario,
        limits: TrialLimits,
        cancellation: &CancellationToken,
    ) -> Result<Self, TrialFailure> {
        let phase = ExecutionPhase::TrialPreparation;
        let mut position = TrialPosition::Initial;
        let effective = limits
            .remaining(0, phase)
            .map_err(|error| TrialFailure::new(position, phase, error))?;
        let _trace_attempt = trace_attempt(limits.trace, phase)
            .map_err(|error| TrialFailure::new(position, phase, error))?;
        let (candidate, usage, native_storage) = control::run_with_provenance(
            &effective,
            cancellation,
            phase,
            NativeStorageProvenance::fresh(),
            || {
                let prefix_len = scenario.preparation_steps().len();
                let mut candidate = Self::initialize_in_scope(content, scenario, limits)?;
                for index in 0..prefix_len {
                    position = TrialPosition::Step { index };
                    // No nested scope or intermediate adoption of usage/ledger.
                    // All headers refer to cumulative usage of this one scope.
                    candidate = candidate.prepare_step(phase)?;
                }
                position = TrialPosition::PreparationReady;
                candidate.finish_preparation_in_scope()?;
                Ok::<_, AuthoringError>(candidate)
            },
        )
        .map_err(|diagnostic| TrialFailure::new(position, phase, execution_error(diagnostic)))?;
        let mut candidate = candidate.map_err(|error| TrialFailure::new(position, phase, error))?;
        candidate.native_storage = native_storage;
        candidate.accepted_work = usage.work;
        Ok(candidate)
    }

    #[cfg(test)]
    pub(super) fn replace_prepared(
        &mut self,
        content: AuthoringContent,
        scenario: TrialScenario,
        limits: TrialLimits,
        cancellation: &CancellationToken,
    ) -> Result<&PreparedReceipt, AuthoringError> {
        let candidate = Self::prepare_owned(content, scenario, limits, cancellation)?;
        *self = candidate;
        Ok(self
            .preparation_receipt
            .as_deref()
            .expect("prepared candidate owns its ready receipt"))
    }

    fn finish_preparation_in_scope(&mut self) -> Result<(), AuthoringError> {
        let phase = ExecutionPhase::TrialPreparation;
        let lifecycle = self.application.authoring_lifecycle().ok_or_else(|| {
            request_error(
                phase,
                "authoring_trial_lifecycle_missing",
                "The private trial application has no lifecycle owner.",
            )
        })?;
        if !lifecycle.is_ready() {
            return Err(request_error(
                phase,
                "authoring_trial_battle_not_prepared",
                "The complete explicit preparation prefix did not retain a completed battle entry.",
            ));
        }
        if lifecycle.total_elapsed_ms > self.scenario.limits().max_total_clock_ms {
            return Err(request_error(
                phase,
                "authoring_trial_clock_limit",
                "Actual cumulative battle time exceeds the scenario ceiling.",
            ));
        }
        control::charge(1, "trial_preparation_receipt_owner").map_err(interrupted)?;
        let mut receipt = Arc::new(empty_receipt());
        #[derive(Serialize)]
        struct Ready<'a> {
            schema_version: u32,
            kind: &'static str,
            usage_basis: &'static str,
            input_sha256: &'a str,
            content: &'a crate::game::data::content_pack::ContentIdentity,
            runtime_safety_sha256: &'a str,
            source_receipt: &'a crate::game::data::content_pack::SourceReceipt,
            preparation_steps: usize,
            prior_accepted_work: u64,
            lifecycle: &'a crate::game::world::authoring_lifecycle::TrialLifecycle,
            observation: crate::game::world::authoring_observation::TrialObservationView<'a>,
        }
        let prepared = output::encode_receipt(
            &Ready {
                schema_version: 1,
                kind: "preparation_ready",
                usage_basis: "scope_cumulative",
                input_sha256: self.scenario.input_sha256(),
                content: self.application.content_identity(),
                runtime_safety_sha256: self.application.runtime_safety().effective_sha256(),
                source_receipt: self.content.token().source_receipt(),
                preparation_steps: self.records.len(),
                prior_accepted_work: self.accepted_work,
                lifecycle: &lifecycle,
                observation: self
                    .application
                    .authoring_observation()
                    .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?,
            },
            self.receipt_bytes,
            self.limits.max_receipt_bytes,
        )?;
        self.receipt_bytes = prepared.total_bytes;
        *Arc::get_mut(&mut receipt).expect("unpublished unique ready receipt") = prepared;
        self.preparation_receipt = Some(receipt);
        Ok(())
    }

    /// Constructs only the initial memory application. This does not claim that
    /// a battle was prepared and is not exposed as public TrialSession::prepare.
    #[cfg(test)]
    pub(super) fn initialize(
        content: AuthoringContent,
        scenario: TrialScenario,
        limits: TrialLimits,
        cancellation: &CancellationToken,
    ) -> Result<Self, AuthoringError> {
        let phase = ExecutionPhase::TrialPreparation;
        let effective = limits.remaining(0, phase)?;
        let _trace_attempt = trace_attempt(limits.trace, phase)?;
        let (candidate, usage, native_storage) = control::run_with_provenance(
            &effective,
            cancellation,
            phase,
            NativeStorageProvenance::fresh(),
            || Self::initialize_in_scope(content, scenario, limits),
        )
        .map_err(execution_error)?;
        // The outer Ok may contain a domain Err. Only complete success receives
        // the candidate ledger; no charged work follows the acceptance poll.
        let mut candidate = candidate?;
        candidate.native_storage = native_storage;
        candidate.accepted_work = usage.work;
        Ok(candidate)
    }

    fn initialize_in_scope(
        content: AuthoringContent,
        scenario: TrialScenario,
        limits: TrialLimits,
    ) -> Result<Self, AuthoringError> {
        let phase = ExecutionPhase::TrialPreparation;
        if !control::is_active() {
            return Err(request_error(
                phase,
                "authoring_trial_scope_missing",
                "Trial construction requires its owning scope.",
            ));
        }
        control::charge(4, "trial_initial_owners").map_err(interrupted)?;
        let content = Arc::new(content);
        let scenario = Arc::new(scenario);
        let token = controlled_clone(content.token()).map_err(interrupted)?;
        let mut application =
            CoreApplication::new_for_authoring(token, scenario.initial().run_seed);
        application.set_authoring_trace_options(limits.trace);
        application.set_amount_observation_enabled(limits.amount_observation);
        let profile = application
            .snapshot(ApplicationSnapshotKind::PlayerProfile)
            .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?;
        let profile = public_value(&profile)?;
        // Allocate the sole receipt owner before final mandatory encoding.
        // Its temporary empty value is private and never published.
        let mut receipt = Arc::new(empty_receipt());
        #[derive(Serialize)]
        struct Initial<'a> {
            schema_version: u32,
            kind: &'static str,
            usage_basis: &'static str,
            initial: &'a super::TrialInitial,
            input_sha256: &'a str,
            input_bytes: usize,
            content: &'a crate::game::data::content_pack::ContentIdentity,
            runtime_safety_sha256: &'a str,
            source_receipt: &'a crate::game::data::content_pack::SourceReceipt,
            validation_usage: &'a ExecutionUsage,
            player_profile: &'a Value,
            prior_accepted_work: u64,
        }
        let prepared = output::encode_receipt(
            &Initial {
                schema_version: 1,
                kind: "initial",
                usage_basis: "scope_cumulative",
                initial: scenario.initial(),
                input_sha256: scenario.input_sha256(),
                input_bytes: scenario.input_bytes(),
                content: application.content_identity(),
                runtime_safety_sha256: application.runtime_safety().effective_sha256(),
                source_receipt: content.token().source_receipt(),
                validation_usage: content.validation_usage(),
                player_profile: &profile,
                prior_accepted_work: 0,
            },
            0,
            limits.max_receipt_bytes,
        )?;
        let receipt_bytes = prepared.total_bytes;
        *Arc::get_mut(&mut receipt).expect("unpublished unique receipt owner") = prepared;
        Ok::<_, AuthoringError>(Self {
            application,
            native_storage: NativeStorageProvenance::fresh(),
            content,
            scenario,
            limits: limits.clone(),
            initial_receipt: receipt,
            preparation_receipt: None,
            records: Vec::new(),
            copied_bytes: 0,
            receipt_bytes,
            accepted_work: 0,
        })
    }

    pub(super) fn advance(
        &mut self,
        cancellation: &CancellationToken,
    ) -> Result<&AcceptedStep, AuthoringError> {
        let index = self.records.len();
        let phase = if index < self.scenario.preparation_steps().len() {
            ExecutionPhase::TrialPreparation
        } else {
            ExecutionPhase::TrialExecution
        };
        if self.scenario.step(index).is_none() {
            return Err(request_error(
                phase,
                "authoring_trial_finished",
                "No finite scenario step remains.",
            ));
        }
        let effective = self.limits.remaining(self.accepted_work, phase)?;
        let _trace_attempt = trace_attempt(self.limits.trace, phase)?;
        let (candidate, usage, native_storage) = control::run_with_provenance(
            &effective,
            cancellation,
            phase,
            self.native_storage,
            || self.prepare_step(phase),
        )
        .map_err(execution_error)?;
        let mut candidate = candidate?;
        candidate.native_storage = native_storage;
        // effective.max_work <= max_total_work - accepted_work, validated above.
        candidate.accepted_work = self.accepted_work + usage.work;
        *self = candidate;
        Ok(self.records.last().expect("one prepared step was appended"))
    }

    fn prepare_step(&self, phase: ExecutionPhase) -> Result<Self, AuthoringError> {
        let index = self.records.len();
        let step = self
            .scenario
            .step(index)
            .expect("checked finite step index");
        let mut application = self
            .application
            .fork_for_authoring()
            .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?;
        // Bounded temporary pointer/Option slots; public Values remain borrowed.
        let reference_slots = index
            .checked_mul(3)
            .and_then(|n| n.checked_add(1))
            .unwrap_or(usize::MAX);
        control::admit(
            Resource::CloneSlots,
            0,
            reference_slots,
            "trial_public_result_view",
        )
        .map_err(interrupted)?;
        control::charge(reference_slots as u64, "trial_public_result_view").map_err(interrupted)?;
        let public_results: Vec<_> = self
            .records
            .iter()
            .map(|record| record.public_result.as_ref())
            .collect();
        let mut copied_bytes = self.copied_bytes;
        let mut resolved_input = None;
        let mut assertion = None;
        let public_result = match step {
            TrialStep::Command { .. } => {
                let resolved =
                    resolve::resolve_command(&self.scenario, index, &public_results, copied_bytes)?;
                let command = public_value(resolved.command.document())?;
                resolved_input = Some(ResolvedInput {
                    command,
                    context: resolved.resolved_context,
                });
                copied_bytes = resolved.total_copy_bytes;
                let result = application
                    .execute(
                        self.scenario.initial().player_id,
                        resolved.command,
                        resolved.context,
                    )
                    .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?;
                Some(public_value(&result)?)
            }
            TrialStep::Snapshot { snapshot } => {
                let result = application
                    .snapshot(*snapshot)
                    .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?;
                Some(public_value(&result)?)
            }
            TrialStep::Clock { clock } => {
                let result = match clock {
                    TrialClock::Elapsed {
                        delta_ms,
                        after_seq,
                    } => application
                        .advance_active_battle_by(*delta_ms, *after_seq)
                        .map(Some),
                    TrialClock::HostTick {
                        delta_ms,
                        after_seq,
                    } => application.advance_active_battle_for_host_tick(*delta_ms, *after_seq),
                    TrialClock::NextEventBucket => application
                        .advance_active_battle_to_next_event_bucket()
                        .map(Some),
                }
                .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?;
                result.as_ref().map(public_value).transpose()?
            }
            TrialStep::Assert { .. } => {
                assertion = Some(resolve::resolve_assertion(
                    &self.scenario,
                    index,
                    &public_results,
                )?);
                None
            }
        };
        drop(public_results);
        let lifecycle = application.authoring_lifecycle().ok_or_else(|| {
            request_error(
                phase,
                "authoring_trial_lifecycle_missing",
                "The private trial application has no lifecycle owner.",
            )
        })?;
        if lifecycle.total_elapsed_ms > self.scenario.limits().max_total_clock_ms {
            return Err(request_error(
                phase,
                "authoring_trial_clock_limit",
                "Actual cumulative battle time exceeds the scenario ceiling.",
            ));
        }

        // Price the new immutable prefix as Vec + existing/new Arc handles. No
        // prior public JSON or receipt is cloned through these shared owners.
        let next_len = index.checked_add(1).unwrap_or(usize::MAX);
        let mut meter = CloneMeter::default();
        meter
            .scan_storage(next_len.checked_add(1))
            .map_err(interrupted)?;
        for record in &self.records {
            record.visit_clone_shape(&mut meter).map_err(interrupted)?;
        }
        meter.scan_storage(Some(1)).map_err(interrupted)?; // New Arc handle.
        meter
            .finish("trial_prefix_storage", "trial_prefix_copy")
            .map_err(interrupted)?;
        let mut records = Vec::with_capacity(next_len);
        records.extend(self.records.iter().cloned());
        let content = controlled_clone(&self.content).map_err(interrupted)?;
        let scenario = controlled_clone(&self.scenario).map_err(interrupted)?;
        let initial_receipt = controlled_clone(&self.initial_receipt).map_err(interrupted)?;
        let preparation_receipt =
            controlled_clone(&self.preparation_receipt).map_err(interrupted)?;
        control::charge(1, "trial_step_record").map_err(interrupted)?;
        let mut record = Arc::new(AcceptedStep {
            public_result,
            receipt: empty_receipt(),
        });
        #[derive(Serialize)]
        struct StepPayload<'a> {
            schema_version: u32,
            kind: &'static str,
            usage_basis: &'static str,
            index: usize,
            phase: ExecutionPhase,
            input_sha256: &'a str,
            content: &'a crate::game::data::content_pack::ContentIdentity,
            runtime_safety_sha256: &'a str,
            source_receipt: &'a crate::game::data::content_pack::SourceReceipt,
            request: &'a TrialStep,
            resolved_input: &'a Option<ResolvedInput>,
            public_result: &'a Option<Value>,
            assertion: &'a Option<resolve::AssertionObservation>,
            prior_accepted_work: u64,
            lifecycle: &'a crate::game::world::authoring_lifecycle::TrialLifecycle,
            observation: crate::game::world::authoring_observation::TrialObservationView<'a>,
        }
        let prepared = output::encode_receipt(
            &StepPayload {
                schema_version: 1,
                kind: "step",
                usage_basis: "scope_cumulative",
                index,
                phase,
                input_sha256: scenario.input_sha256(),
                content: application.content_identity(),
                runtime_safety_sha256: application.runtime_safety().effective_sha256(),
                source_receipt: content.token().source_receipt(),
                request: step,
                resolved_input: &resolved_input,
                public_result: &record.public_result,
                assertion: &assertion,
                prior_accepted_work: self.accepted_work,
                lifecycle: &lifecycle,
                observation: application
                    .authoring_observation()
                    .map_err(|diagnostic| AuthoringError::Application { phase, diagnostic })?,
            },
            self.receipt_bytes,
            self.limits.max_receipt_bytes,
        )?;
        let receipt_bytes = prepared.total_bytes;
        Arc::get_mut(&mut record)
            .expect("unpublished unique step owner")
            .receipt = prepared;
        records.push(record); // Capacity and every shared handle were prepared.
        Ok(Self {
            application,
            native_storage: self.native_storage,
            content,
            scenario,
            limits: self.limits.clone(),
            initial_receipt,
            preparation_receipt,
            records,
            copied_bytes,
            receipt_bytes,
            accepted_work: self.accepted_work,
        })
    }
}

fn trace_attempt(
    options: super::trace::TraceOptions,
    phase: ExecutionPhase,
) -> Result<super::trace::TraceAttempt, AuthoringError> {
    super::trace::TraceAttempt::enter(options).map_err(|()| {
        request_error(
            phase,
            "authoring_trace_scope_invalid",
            "Optional trace requires a valid, nonnested attempt owner.",
        )
    })
}

fn public_value(value: &impl Serialize) -> Result<Value, AuthoringError> {
    let _phase = control::phase(ExecutionPhase::Receipt);
    super::projection::to_value(value).map_err(|_| {
        request_error(
            ExecutionPhase::Receipt,
            "authoring_result_encoding",
            "The admitted public document could not be materialized.",
        )
    })
}

fn empty_receipt() -> PreparedReceipt {
    PreparedReceipt {
        document: String::new(),
        total_bytes: 0,
    }
}

fn execution_error(diagnostic: super::ExecutionFailure) -> AuthoringError {
    AuthoringError::Execution { diagnostic }
}

fn interrupted(_: crate::game::behavior::GameError) -> AuthoringError {
    request_error(
        ExecutionPhase::Receipt,
        "authoring_trial_interrupted",
        "The attempt owns the latched execution diagnostic.",
    )
}

fn request_error(
    phase: ExecutionPhase,
    code: &'static str,
    reason: &'static str,
) -> AuthoringError {
    AuthoringError::Request {
        diagnostic: RequestFailure {
            code,
            phase,
            field_path: "trial",
            reason: reason.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    mod amount_completion;
    mod amount_order;
    mod r07_copies;
    mod u07_objects;
    mod u08_relics;
    use super::*;
    use crate::game::data::{content_pack::tests::ExternalFixture, ValidatedGameData};
    use serde_json::json;

    fn fixture_content(token: &ValidatedGameData) -> AuthoringContent {
        AuthoringContent {
            token: token.clone(),
            validation_usage: ExecutionUsage::default(),
        }
    }

    // Test authoring only: select a finite set of public array indices once.
    // The actual runner receives the resulting explicit JSON; it cannot search
    // candidates, choose nodes, or append commands on the user's behalf.
    fn preparation_document(token: &ValidatedGameData) -> Value {
        use crate::application::{ApplicationCommand, ApplicationDocument};
        let player = uuid::Uuid::from_u128(25);
        let mut app = CoreApplication::new(token.clone(), 77);
        fn command(app: &mut CoreApplication, player: uuid::Uuid, input: Value) -> Value {
            let command =
                ApplicationCommand::from_document(ApplicationDocument::new(input)).unwrap();
            serde_json::to_value(app.execute(player, command, Default::default()).unwrap()).unwrap()
        }
        let start = command(
            &mut app,
            player,
            json!({"type":"start_new_game", "game_mode":"Standard", "replace_active_run":false}),
        );
        command(
            &mut app,
            player,
            json!({"type":"advance_visual_novel", "session_id":start["payload"]["visual_novel"]["session_id"], "current_scene_id":start["payload"]["visual_novel"]["current_scene_id"]}),
        );
        let snapshot =
            serde_json::to_value(app.snapshot(ApplicationSnapshotKind::Run).unwrap()).unwrap();
        let context = &snapshot["read_model"]["game_state_context"];
        let count = context["required_count"].as_u64().unwrap() as usize;
        let ids: Vec<_> = context["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .take(count)
            .map(|entry| entry["employee_definition_id"].clone())
            .collect();
        assert_eq!(ids.len(), count);
        let selected = command(
            &mut app,
            player,
            json!({"type":"select_starter_employees", "employee_definition_ids":ids}),
        );
        let node_index = selected["payload"]["map"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .position(|node| node["state"] == "Available" && node["category"] == "Combat")
            .unwrap();
        let starter_bindings: Vec<_> = (0..count).map(|index| json!({
            "from":{"step":2,"pointer":format!("/read_model/game_state_context/candidates/{index}/employee_definition_id")},
            "to":format!("/command/employee_definition_ids/{index}"), "value_type":"string"
        })).collect();
        json!({
            "initial":{"player_id":player, "run_seed":77, "profile":"default_memory"},
            "preparation_steps":[
                {"kind":"command", "command":{"type":"start_new_game", "game_mode":"Standard", "replace_active_run":false}},
                {"kind":"command", "command":{"type":"advance_visual_novel", "session_id":null,"current_scene_id":null}, "bindings":[
                    {"from":{"step":0,"pointer":"/payload/visual_novel/session_id"},"to":"/command/session_id","value_type":"string"},
                    {"from":{"step":0,"pointer":"/payload/visual_novel/current_scene_id"},"to":"/command/current_scene_id","value_type":"string"}
                ]},
                {"kind":"snapshot", "snapshot":"run"},
                {"kind":"command", "command":{"type":"select_starter_employees","employee_definition_ids":vec![Value::Null;count]}, "bindings":starter_bindings},
                {"kind":"command", "command":{"type":"select_map_node","node_id":null},"bindings":[
                    {"from":{"step":3,"pointer":format!("/payload/map/nodes/{node_index}/id")},"to":"/command/node_id","value_type":"string"}
                ]},
                {"kind":"command", "command":{"type":"confirm_enter_node"}},
                authority_step("pause_battle",5),
                {"kind":"clock","clock":{"mode":"host_tick","delta_ms":1,"after_seq":0}},
                {"kind":"assert","from":{"step":5,"pointer":"/kind"},"assertion":{"test":"type","value_type":"string"}}
            ],
            "steps":[{"kind":"snapshot","snapshot":"run"},{"kind":"clock","clock":{"mode":"next_event_bucket"}}]
        })
    }

    fn authority_step(kind: &str, source: usize) -> Value {
        json!({"kind":"command", "command":{"type":kind}, "context":{"expected_battle_authority":{"battle_uuid":null,"battle_state_revision":null}},"bindings":[
            {"from":{"step":source,"pointer":"/payload/battle_uuid"},"to":"/context/expected_battle_authority/battle_uuid","value_type":"string"},
            {"from":{"step":source,"pointer":"/payload/battle_update/battle_state_revision"},"to":"/context/expected_battle_authority/battle_state_revision","value_type":"number"}
        ]})
    }

    fn checked(document: &Value) -> TrialScenario {
        TrialScenario::from_json_slice(&serde_json::to_vec(document).unwrap(), &Default::default())
            .unwrap()
    }

    #[test]
    fn public_trial_preview_is_bounded_and_preserves_the_accepted_session() {
        use crate::{
            application::{ApplicationCommand, ApplicationDocument, ApplicationExecutionContext},
            authoring::TrialPreview,
            game::{ability::SkillId, resources::board::Position},
        };
        let fixture = ExternalFixture::official_with_legacy_starters();
        let token = fixture.load();
        let document = preparation_document(&token);
        let cancel = CancellationToken::default();
        let mut session = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        let explain_site = token
            .value_expr_catalog()
            .entries()
            .next()
            .unwrap()
            .0
            .clone();
        let explain_query = super::super::ExplainQuery::Effect { site: explain_site };
        let direct_explanation = super::super::explain(
            &fixture_content(&token),
            &explain_query,
            &ExecutionLimits::default(),
            &CancellationToken::default(),
        )
        .unwrap();
        let delegated_explanation = session
            .explain(
                &explain_query,
                &ExecutionLimits::default(),
                &CancellationToken::default(),
            )
            .unwrap();
        assert_eq!(
            delegated_explanation.document(),
            direct_explanation.document()
        );
        assert_eq!(delegated_explanation.usage(), direct_explanation.usage());
        let employee_uuid = serde_json::from_value(
            session.step(6).unwrap().public_result().unwrap()["payload"]["battle_update"]
                ["checkpoint"]["deployment"]["unit_deploy_costs"][0]["employee_uuid"]
                .clone(),
        )
        .unwrap();
        let zones = session.step(4).unwrap().public_result().unwrap()["payload"]["combat_preview"]
            ["deployment_zones"]
            .as_array()
            .unwrap();
        let position: Position = serde_json::from_value(
            zones.iter().find(|zone| zone["kind"] == "Ground").unwrap()["cells"][0].clone(),
        )
        .unwrap();
        let request = || TrialPreview::RequestDeploymentRangePreview {
            employee_uuid,
            position,
        };
        let context = || ApplicationExecutionContext {
            source_command_id: Some("preview\n호출".into()),
            battle_event_after_seq: 0,
            expected_battle_authority: None,
        };
        let limits = ExecutionLimits::default();
        let before = session
            .receipt_documents()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let before_work = session.accepted_work();
        // Independent ordinary public decode and execute on a fixture-only fork.
        let mut direct = session.state.application.fork_for_authoring().unwrap();
        let command = ApplicationCommand::from_document(ApplicationDocument::new(
            serde_json::to_value(request()).unwrap(),
        ))
        .unwrap();
        let expected = serde_json::to_value(
            direct
                .execute(session.initial().player_id, command, context())
                .unwrap(),
        )
        .unwrap();
        for _ in 0..3 {
            let result = session
                .preview(request(), context(), &limits, &cancel)
                .unwrap();
            let actual: Value = serde_json::from_str(result.document()).unwrap();
            assert_eq!(actual, expected);
            assert_eq!(actual["kind"], "deployment_range_preview");
        }
        // An undeployed employee cannot gain a targeting focus through Preview.
        let failure = session
            .preview(
                TrialPreview::PreviewSkillTarget {
                    employee_uuid,
                    skill_id: SkillId::new("preview-only"),
                    target: crate::game::battle::event_log::SkillCastTarget::Tile { position },
                },
                context(),
                &limits,
                &cancel,
            )
            .unwrap_err();
        assert!(matches!(
            failure,
            AuthoringError::Application {
                phase: ExecutionPhase::Inspection,
                ..
            }
        ));
        for resource in 0..3 {
            let mut limited = limits.clone();
            match resource {
                0 => limited.max_work = 1,
                1 => limited.max_result_bytes = 1,
                _ => limited.max_clone_slots = 1,
            }
            assert!(
                matches!(session.preview(request(), context(), &limited, &cancel),
                Err(AuthoringError::Execution { diagnostic }) if diagnostic.phase == ExecutionPhase::Inspection)
            );
        }
        let mut small = limits.clone();
        small.max_metadata_bytes = 256;
        let error = session
            .preview(request(), context(), &small, &cancel)
            .unwrap_err();
        assert!(matches!(error, AuthoringError::Execution { diagnostic }
            if diagnostic.site == "preview_input_bytes" && diagnostic.phase == ExecutionPhase::Inspection));
        let cancelled = CancellationToken::default();
        cancelled.cancel();
        assert!(matches!(
            session.preview(request(), context(), &limits, &cancelled),
            Err(AuthoringError::Execution { .. })
        ));
        assert_eq!(session.receipt_documents().collect::<Vec<_>>(), before);
        assert_eq!(session.accepted_work(), before_work);
        assert_eq!(session.next_step_index(), 9);
        session.run_to_end(&cancel).unwrap();
        let replay = simulate(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        assert_eq!(
            session.receipt_documents().collect::<Vec<_>>(),
            replay.receipt_documents().collect::<Vec<_>>()
        );
    }

    #[test]
    fn public_trial_session_preserves_receipt_order_none_slots_and_finite_completion() {
        let token = ValidatedGameData::load_live_embedded().unwrap();
        let document = preparation_document(&token);
        let cancel = CancellationToken::default();
        let mut session = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        assert_eq!(session.next_step_index(), 9);
        assert!(session.step(7).unwrap().public_result().is_none());
        assert!(session.step(8).unwrap().public_result().is_none());
        assert!(session.step(9).is_none());
        assert!(!session.is_complete());
        let ready: Value = serde_json::from_str(session.ready_receipt().document()).unwrap();
        let initial_work = session.accepted_work();
        let limits = ExecutionLimits::default();
        let snapshot = session
            .snapshot(ApplicationSnapshotKind::Run, &limits, &cancel)
            .unwrap();
        for _ in 0..3 {
            let observed = session.observation().unwrap();
            assert_eq!(
                serde_json::to_value(&observed).unwrap(),
                ready["payload"]["observation"]
            );
            let active = observed.active().unwrap();
            assert_eq!(
                active.key().unwrap(),
                session.lifecycle().current().unwrap().key()
            );
            assert!(session.lifecycle().is_ready());
            let trace = session.trace().unwrap();
            assert_eq!(trace.active().unwrap().key(), active.key().unwrap());
            assert!(!trace.active().unwrap().enabled());
            assert!(trace.active().unwrap().ndjson().is_empty());
            assert_eq!(
                session
                    .snapshot(ApplicationSnapshotKind::Run, &limits, &cancel)
                    .unwrap()
                    .document(),
                snapshot.document()
            );
            assert!(!session
                .reconnect(0, &limits, &cancel)
                .unwrap()
                .document()
                .is_empty());
        }
        let mut too_small = limits.clone();
        too_small.max_result_bytes = 1;
        assert!(matches!(
            session.snapshot(ApplicationSnapshotKind::Run, &too_small, &cancel),
            Err(AuthoringError::Execution { .. })
        ));
        let cancelled = CancellationToken::default();
        cancelled.cancel();
        assert!(matches!(
            session.reconnect(0, &limits, &cancelled),
            Err(AuthoringError::Execution { .. })
        ));
        assert_eq!(session.accepted_work(), initial_work);
        assert_eq!(session.next_step_index(), 9);
        session.run_to_end(&cancel).unwrap();
        assert!(session.is_complete());
        assert_eq!(session.next_step_index(), 11);
        let documents: Vec<_> = session.receipt_documents().collect();
        assert_eq!(documents.len(), 13);
        assert_eq!(documents[0], session.initial_receipt().document());
        for index in 0..9 {
            assert_eq!(
                documents[index + 1],
                session.step(index).unwrap().receipt().document()
            );
        }
        assert_eq!(documents[10], session.ready_receipt().document());
        for index in 9..11 {
            assert_eq!(
                documents[index + 2],
                session.step(index).unwrap().receipt().document()
            );
        }
        assert_eq!(session.steps().len(), 11);
        let work = session.accepted_work();
        let failure = session.advance(&cancel).unwrap_err();
        assert_eq!(failure.position(), TrialPosition::End);
        assert_eq!(failure.operation_phase(), ExecutionPhase::TrialExecution);
        assert_eq!(
            error_code::<()>(Err(failure.into_error())),
            "authoring_trial_finished"
        );
        session.run_to_end(&cancel).unwrap();
        assert_eq!(session.accepted_work(), work);
        let replay = simulate(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        assert_eq!(
            session.receipt_documents().collect::<Vec<_>>(),
            replay.receipt_documents().collect::<Vec<_>>()
        );
    }

    #[test]
    fn public_trial_failures_keep_preparation_position_and_partial_simulation_owner() {
        let token = ValidatedGameData::load_live_embedded().unwrap();
        let document = preparation_document(&token);
        let cancel = CancellationToken::default();
        let cancelled = CancellationToken::default();
        cancelled.cancel();
        let error = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancelled,
        )
        .unwrap_err();
        assert_eq!(error.position(), TrialPosition::Initial);
        assert_eq!(
            error_code::<()>(Err(error.into_error())),
            "authoring_cancelled"
        );
        let mut invalid = document.clone();
        invalid["preparation_steps"][1] =
            json!({"kind":"command", "command":{"type":"unknown_command"}});
        let error = simulate(
            fixture_content(&token),
            checked(&invalid),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap_err();
        assert!(error.accepted().is_none());
        assert_eq!(error.failure().position(), TrialPosition::Step { index: 1 });
        assert_eq!(
            error.failure().operation_phase(),
            ExecutionPhase::TrialPreparation
        );
        assert!(matches!(
            error.failure().error(),
            AuthoringError::Application { .. }
        ));
        let mut no_entry = document.clone();
        no_entry["preparation_steps"] = json!([]);
        let error = TrialSession::prepare(
            fixture_content(&token),
            checked(&no_entry),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap_err();
        assert_eq!(error.position(), TrialPosition::PreparationReady);

        let mut partial = document.clone();
        partial["steps"] = json!([
            {"kind":"snapshot", "snapshot":"run"},
            {"kind":"command", "command":{"type":"unknown_command"}},
            {"kind":"snapshot", "snapshot":"run"}
        ]);
        let error = simulate(
            fixture_content(&token),
            checked(&partial),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap_err();
        assert_eq!(
            error.failure().position(),
            TrialPosition::Step { index: 10 }
        );
        assert_eq!(error.accepted().unwrap().next_step_index(), 10);
        let render_limits = super::super::DiagnosticLimits::default();
        let rendered =
            super::super::render_trial_failure(error.failure(), &render_limits, &cancel).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(rendered.document()).unwrap(),
            serde_json::to_value(error.failure()).unwrap()
        );
        let mut render_small = render_limits;
        render_small.max_output_bytes = 1;
        assert_eq!(
            super::super::render_trial_failure(error.failure(), &render_small, &cancel)
                .unwrap_err(),
            super::super::DiagnosticRenderError::OutputBytes
        );
        assert_eq!(error.accepted().unwrap().next_step_index(), 10);
        let (session, failure) = error.into_parts();
        let mut session = session.unwrap();
        assert_eq!(failure.operation_phase(), ExecutionPhase::TrialExecution);
        let before: Vec<_> = session.receipt_documents().map(str::to_owned).collect();
        let before_work = session.accepted_work();
        let before_input = session.input_sha256().to_owned();
        for _ in 0..2 {
            assert_eq!(
                session.advance(&cancel).unwrap_err().position(),
                TrialPosition::Step { index: 10 }
            );
            assert_eq!(
                session
                    .replace(
                        fixture_content(&token),
                        checked(&invalid),
                        TrialLimits::default(),
                        &cancel
                    )
                    .unwrap_err()
                    .position(),
                TrialPosition::Step { index: 1 }
            );
            assert_eq!(session.receipt_documents().collect::<Vec<_>>(), before);
            assert_eq!(session.accepted_work(), before_work);
            assert_eq!(session.input_sha256(), before_input);
        }
        let mut replacement = document.clone();
        replacement["initial"]["run_seed"] = json!(78);
        session
            .replace(
                fixture_content(&token),
                checked(&replacement),
                TrialLimits::default(),
                &cancel,
            )
            .unwrap();
        assert_eq!(session.next_step_index(), 9);
        assert_eq!(session.initial().run_seed, 78);
        assert_ne!(session.input_sha256(), before_input);
    }

    #[test]
    fn optional_trace_private_trial_keeps_complete_receipts_and_public_results_identical() {
        use super::super::trace::{TraceLimits, TraceOptions};
        let fixture = ExternalFixture::official_with_legacy_starters();
        let token = fixture.load();
        let mut document = preparation_document(&token);
        let cancel = CancellationToken::default();
        // Fixture-only selection from an actual public preview. The production
        // runner receives the resulting explicit finite command and bindings.
        let probe = TrialState::prepare_owned(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        let zones = probe.records[4].public_result.as_ref().unwrap()["payload"]["combat_preview"]
            ["deployment_zones"]
            .as_array()
            .unwrap();
        let ground = zones
            .iter()
            .position(|zone| zone["kind"] == "Ground")
            .unwrap();
        let mut deploy = authority_step("deploy_unit", 6);
        deploy["command"]["employee_uuid"] = Value::Null;
        deploy["command"]["position"] = Value::Null;
        deploy["command"]["facing"] = json!("right");
        deploy["bindings"].as_array_mut().unwrap().extend([
            json!({"from":{"step":6,"pointer":"/payload/battle_update/checkpoint/deployment/unit_deploy_costs/0/employee_uuid"},
                "to":"/command/employee_uuid", "value_type":"string"}),
            json!({"from":{"step":4,"pointer":format!("/payload/combat_preview/deployment_zones/{ground}/cells/0")},
                "to":"/command/position", "value_type":"object"}),
        ]);
        drop(probe);

        document["steps"] = Value::Array(
            std::iter::once(deploy)
                .chain((0..64).map(|_| {
                    json!({
                        "kind":"clock", "clock":{"mode":"elapsed", "delta_ms":1000, "after_seq":0}
                    })
                }))
                .collect(),
        );
        let mut states: Vec<_> = [
            None,
            Some(TraceLimits::default()),
            Some(TraceLimits {
                max_bytes: 1,
                max_row_bytes: 1,
                ..Default::default()
            }),
        ]
        .into_iter()
        .map(|limits| {
            TrialState::prepare_owned(
                fixture_content(&token),
                checked(&document),
                TrialLimits {
                    trace: TraceOptions { limits },
                    ..Default::default()
                },
                &cancel,
            )
            .unwrap()
        })
        .collect();
        let compare = |states: &[TrialState]| {
            for other in &states[1..] {
                assert_eq!(
                    states[0].initial_receipt.document,
                    other.initial_receipt.document
                );
                assert_eq!(
                    states[0].preparation_receipt.as_ref().unwrap().document,
                    other.preparation_receipt.as_ref().unwrap().document
                );
                assert_eq!(states[0].accepted_work, other.accepted_work);
                assert_eq!(states[0].copied_bytes, other.copied_bytes);
                assert_eq!(states[0].receipt_bytes, other.receipt_bytes);
                assert_eq!(
                    states[0].application.authoring_lifecycle(),
                    other.application.authoring_lifecycle()
                );
                assert_eq!(states[0].records.len(), other.records.len());
                for (a, b) in states[0].records.iter().zip(&other.records) {
                    assert_eq!(a.public_result, b.public_result);
                    assert_eq!(a.receipt.document, b.receipt.document);
                }
            }
        };
        compare(&states);
        let applications = |state: &TrialState| {
            let view = state.application.authoring_observation().unwrap();
            view.active
                .or_else(|| view.last_terminal.map(|capsule| &capsule.mandatory))
                .unwrap()
                .applications
        };
        for _ in 0..65 {
            if applications(&states[0]) > 0 {
                break;
            }
            for state in &mut states {
                state.advance(&cancel).unwrap();
            }
            compare(&states);
        }
        assert!(
            applications(&states[0]) > 0,
            "real fixture must mutate HP/Stats"
        );
        let view = states[1].application.authoring_trace().unwrap();
        let trace = view.active.or(view.last_terminal).unwrap().1;
        assert!(trace.enabled);
        assert!(trace.retained_rows > 0);
        assert!(trace.gap.is_none());
        let view = states[2].application.authoring_trace().unwrap();
        let trace = view.active.or(view.last_terminal).unwrap().1;
        assert!(trace.enabled);
        assert!(trace.ndjson.is_empty());
        assert!(trace.gap.is_some());

        // Use the actual next step's disabled costs as an exact admission oracle.
        let prior_work = states[0].accepted_work;
        states[0].advance(&cancel).unwrap();
        let exact_work = states[0].accepted_work - prior_work;
        let exact_bytes = states[0].receipt_bytes;
        let before_observation =
            serde_json::to_value(states[1].application.authoring_observation().unwrap()).unwrap();
        let before_trace = states[1]
            .application
            .authoring_trace()
            .unwrap()
            .active
            .unwrap()
            .1
            .ndjson
            .to_vec();
        let prefix = Arc::clone(states[1].records.last().unwrap());
        for receipt_failure in [true, false] {
            states[1].limits.max_receipt_bytes = if receipt_failure {
                exact_bytes - 1
            } else {
                TrialLimits::default().max_receipt_bytes
            };
            states[1].limits.execution.max_work = if receipt_failure {
                exact_work
            } else {
                exact_work - 1
            };
            assert_eq!(
                error_code(states[1].advance(&cancel)),
                if receipt_failure {
                    "authoring_receipt_limit"
                } else {
                    "authoring_work_limit"
                }
            );
            assert_eq!(states[1].accepted_work, prior_work);
            assert!(Arc::ptr_eq(states[1].records.last().unwrap(), &prefix));
            assert_eq!(
                serde_json::to_value(states[1].application.authoring_observation().unwrap())
                    .unwrap(),
                before_observation
            );
            assert_eq!(
                states[1]
                    .application
                    .authoring_trace()
                    .unwrap()
                    .active
                    .unwrap()
                    .1
                    .ndjson,
                before_trace
            );
        }
        for state in &mut states[1..] {
            state.limits.max_receipt_bytes = exact_bytes;
            state.limits.execution.max_work = exact_work;
            state.advance(&cancel).unwrap();
        }
        compare(&states);
    }

    #[test]
    fn private_trial_preparation_uses_one_scope_keeps_none_slots_and_rejects_late_output() {
        let token = ValidatedGameData::load_live_embedded().unwrap();
        let document = preparation_document(&token);
        let cancel = CancellationToken::default();
        let mut state = TrialState::prepare_owned(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        assert!(state.application.authoring_lifecycle().unwrap().is_ready());
        assert_eq!(state.records.len(), 9);
        assert!(state.records[7].public_result.is_none()); // Actual paused tick.
        assert!(state.records[8].public_result.is_none()); // Explicit assertion.
        assert!(state.copied_bytes > 0);
        let mut last_work = 0;
        for receipt in std::iter::once(state.initial_receipt.as_ref())
            .chain(state.records.iter().map(|step| &step.receipt))
            .chain(std::iter::once(
                state.preparation_receipt.as_deref().unwrap(),
            ))
        {
            let value: Value = serde_json::from_str(&receipt.document).unwrap();
            assert_eq!(value["payload"]["prior_accepted_work"], 0);
            assert_eq!(value["payload"]["usage_basis"], "scope_cumulative");
            let work = value["usage"]["work"].as_u64().unwrap();
            assert!(work > last_work);
            last_work = work;
        }
        assert_eq!(state.accepted_work, last_work);
        let ready = Arc::clone(state.preparation_receipt.as_ref().unwrap());
        let work = state.accepted_work;
        state.advance(&cancel).unwrap();
        let value: Value = serde_json::from_str(&state.records[9].receipt.document).unwrap();
        assert_eq!(value["payload"]["index"], 9);
        assert_eq!(value["payload"]["prior_accepted_work"], work);
        assert_eq!(
            state.accepted_work,
            work + value["usage"]["work"].as_u64().unwrap()
        );
        assert!(Arc::ptr_eq(
            &ready,
            state.preparation_receipt.as_ref().unwrap()
        ));

        let before = state.application.authoring_lifecycle();
        let copied = state.copied_bytes;
        let receipt_bytes = state.receipt_bytes;
        let work = state.accepted_work;
        let mut replacement_document = document.clone();
        replacement_document["initial"]["run_seed"] = json!(78);
        let replacement = TrialState::prepare_owned(
            fixture_content(&token),
            checked(&replacement_document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        for low_receipt in [true, false] {
            let mut limits = TrialLimits::default();
            if low_receipt {
                limits.max_receipt_bytes = replacement.receipt_bytes - 1;
            } else {
                limits.execution.max_work = replacement.accepted_work - 1;
            }
            let error = state.replace_prepared(
                fixture_content(&token),
                checked(&replacement_document),
                limits,
                &cancel,
            );
            assert_eq!(
                error_code(error),
                if low_receipt {
                    "authoring_receipt_limit"
                } else {
                    "authoring_work_limit"
                }
            );
            assert_eq!(state.application.authoring_lifecycle(), before);
            assert_eq!(state.copied_bytes, copied);
            assert_eq!(state.receipt_bytes, receipt_bytes);
            assert_eq!(state.accepted_work, work);
            assert_eq!(state.records.len(), 10);
            assert!(Arc::ptr_eq(
                &ready,
                state.preparation_receipt.as_ref().unwrap()
            ));
        }
        let mut exact = TrialLimits::default();
        exact.execution.max_work = replacement.accepted_work;
        state
            .replace_prepared(
                fixture_content(&token),
                checked(&replacement_document),
                exact,
                &cancel,
            )
            .unwrap();
        assert_eq!(
            state.preparation_receipt.as_ref().unwrap().document,
            replacement.preparation_receipt.as_ref().unwrap().document
        );
        assert_eq!(
            state.application.authoring_lifecycle(),
            replacement.application.authoring_lifecycle()
        );
        assert_eq!(state.scenario.initial().run_seed, 78);
        assert_eq!(state.records.len(), 9);
    }

    #[test]
    fn private_trial_preparation_and_execution_enforce_actual_clock_and_final_readiness() {
        let token = ValidatedGameData::load_live_embedded().unwrap();
        let document = preparation_document(&token);
        let mut scenario_limits = super::super::ScenarioLimits::default();
        scenario_limits.max_total_clock_ms = 1;
        let scenario = |document: &Value| {
            TrialScenario::from_json_slice(&serde_json::to_vec(document).unwrap(), &scenario_limits)
                .unwrap()
        };
        let cancel = CancellationToken::default();
        let mut state = TrialState::prepare_owned(
            fixture_content(&token),
            scenario(&document),
            TrialLimits::default(),
            &cancel,
        )
        .unwrap();
        state.advance(&cancel).unwrap();
        let ready = Arc::clone(state.preparation_receipt.as_ref().unwrap());
        let before = state.application.authoring_lifecycle();
        let work = state.accepted_work;
        let error = state.advance(&cancel);
        assert!(matches!(
            error,
            Err(AuthoringError::Request {
                diagnostic: RequestFailure {
                    code: "authoring_trial_clock_limit",
                    phase: ExecutionPhase::TrialExecution,
                    ..
                }
            })
        ));
        assert_eq!(state.application.authoring_lifecycle(), before);
        assert_eq!(state.records.len(), 10);
        assert_eq!(state.accepted_work, work);
        let mut too_far = document.clone();
        too_far["preparation_steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"clock","clock":{"mode":"next_event_bucket"}}));
        let error = state.replace_prepared(
            fixture_content(&token),
            scenario(&too_far),
            TrialLimits::default(),
            &cancel,
        );
        assert!(matches!(
            error,
            Err(AuthoringError::Request {
                diagnostic: RequestFailure {
                    code: "authoring_trial_clock_limit",
                    phase: ExecutionPhase::TrialPreparation,
                    ..
                }
            })
        ));
        let mut exited = document.clone();
        exited["preparation_steps"]
            .as_array_mut()
            .unwrap()
            .push(authority_step("retreat_battle", 6));
        assert_eq!(
            error_code(state.replace_prepared(
                fixture_content(&token),
                checked(&exited),
                TrialLimits::default(),
                &cancel
            )),
            "authoring_trial_battle_not_prepared"
        );
        let mut empty = document.clone();
        empty["preparation_steps"] = json!([]);
        assert_eq!(
            error_code(state.replace_prepared(
                fixture_content(&token),
                checked(&empty),
                TrialLimits::default(),
                &cancel
            )),
            "authoring_trial_battle_not_prepared"
        );
        assert_eq!(state.application.authoring_lifecycle(), before);
        assert_eq!(state.accepted_work, work);
        assert_eq!(state.records.len(), 10);
        assert!(Arc::ptr_eq(
            &ready,
            state.preparation_receipt.as_ref().unwrap()
        ));
    }

    fn scenario() -> TrialScenario {
        let document = json!({
            "initial": {"player_id":"00000000-0000-0000-0000-000000000019", "run_seed":77, "profile":"default_memory"},
            "preparation_steps": [],
            "steps": [
                {"kind":"command", "command":{"type":"start_new_game", "game_mode":"Standard", "replace_active_run":false}},
                {"kind":"assert", "from":{"step":0,"pointer":"/kind"}, "assertion":{"test":"type","value_type":"string"}},
                {"kind":"snapshot", "snapshot":"run"},
                {"kind":"clock", "clock":{"mode":"elapsed","delta_ms":1,"after_seq":0}}
            ]
        });
        TrialScenario::from_json_slice(&serde_json::to_vec(&document).unwrap(), &Default::default())
            .unwrap()
    }

    fn initial(token: &ValidatedGameData) -> TrialState {
        // This fixture's zero validation usage is not a measured external load.
        // These tests exercise the private application/receipt boundary only.
        TrialState::initialize(
            AuthoringContent {
                token: token.clone(),
                validation_usage: ExecutionUsage::default(),
            },
            scenario(),
            TrialLimits::default(),
            &CancellationToken::default(),
        )
        .unwrap()
    }

    fn error_code<T>(result: Result<T, AuthoringError>) -> String {
        match result {
            Err(AuthoringError::Request { diagnostic }) => diagnostic.code.to_owned(),
            Err(AuthoringError::Execution { diagnostic }) => diagnostic.code.to_owned(),
            Err(AuthoringError::Application { diagnostic, .. }) => diagnostic.code().to_owned(),
            Err(other) => panic!("unexpected {other}"),
            Ok(_) => panic!("expected failure"),
        }
    }

    #[test]
    fn private_trial_step_receipt_failure_discards_real_start_and_retries_identically() {
        let token = ValidatedGameData::load_live_embedded().unwrap();
        let mut measured = initial(&token);
        let initial_work = measured.accepted_work;
        measured.advance(&CancellationToken::default()).unwrap();
        let step_work = measured.accepted_work - initial_work;
        let expected = &measured.records[0];
        assert!(expected.public_result.is_some());
        let parsed: Value = serde_json::from_str(&expected.receipt.document).unwrap();
        assert_eq!(parsed["usage"]["work"].as_u64(), Some(step_work));
        assert_eq!(
            parsed["payload"]["resolved_input"]["command"]["type"],
            "start_new_game"
        );
        assert_eq!(
            parsed["payload"]["prior_accepted_work"].as_u64(),
            Some(initial_work)
        );

        let mut candidate = initial(&token);
        let original_receipt = Arc::clone(&candidate.initial_receipt);
        let before = format!(
            "{:?}",
            candidate.application.snapshot(ApplicationSnapshotKind::Run)
        );
        candidate.limits.max_receipt_bytes = measured.receipt_bytes - 1;
        assert_eq!(
            error_code(candidate.advance(&CancellationToken::default())),
            "authoring_receipt_limit"
        );
        assert!(candidate.records.is_empty());
        assert_eq!(candidate.accepted_work, initial_work);
        assert_eq!(candidate.copied_bytes, 0);
        assert!(Arc::ptr_eq(&candidate.initial_receipt, &original_receipt));
        assert_eq!(
            format!(
                "{:?}",
                candidate.application.snapshot(ApplicationSnapshotKind::Run)
            ),
            before
        );
        candidate.limits.max_receipt_bytes = TrialLimits::default().max_receipt_bytes;
        candidate.advance(&CancellationToken::default()).unwrap();
        assert_eq!(candidate.records[0].public_result, expected.public_result);
        assert_eq!(
            candidate.records[0].receipt.document,
            expected.receipt.document
        );

        let mut exact = initial(&token);
        exact.limits.execution.max_work = step_work;
        exact.advance(&CancellationToken::default()).unwrap();
        assert_eq!(exact.records[0].receipt.document, expected.receipt.document);
        let mut low = initial(&token);
        low.limits.execution.max_work = step_work - 1;
        assert_eq!(
            error_code(low.advance(&CancellationToken::default())),
            "authoring_work_limit"
        );
        assert!(low.records.is_empty());
        assert_eq!(low.accepted_work, initial_work);
    }

    #[test]
    fn private_trial_prefix_keeps_public_slots_and_survives_domain_and_control_failure() {
        let token = ValidatedGameData::load_live_embedded().unwrap();
        let mut state = initial(&token);
        state.advance(&CancellationToken::default()).unwrap();
        let first = Arc::clone(&state.records[0]);
        state.advance(&CancellationToken::default()).unwrap();
        assert!(state.records[1].public_result.is_none()); // Assert occupies index 1.
        state.advance(&CancellationToken::default()).unwrap();
        assert!(state.records[2].public_result.is_some());
        assert!(Arc::ptr_eq(&first, &state.records[0]));
        let before = format!(
            "{:?}",
            state.application.snapshot(ApplicationSnapshotKind::Run)
        );
        let receipt_bytes = state.receipt_bytes;
        let work = state.accepted_work;
        let failed = state.advance(&CancellationToken::default());
        assert!(matches!(failed, Err(AuthoringError::Application { .. }))); // No actual live battle.
        assert_eq!(state.records.len(), 3);
        assert_eq!(state.accepted_work, work);
        assert_eq!(state.receipt_bytes, receipt_bytes);
        assert_eq!(
            format!(
                "{:?}",
                state.application.snapshot(ApplicationSnapshotKind::Run)
            ),
            before
        );
        let cancellation = CancellationToken::default();
        cancellation.cancel();
        assert_eq!(
            error_code(state.advance(&cancellation)),
            "authoring_cancelled"
        );
        assert_eq!(state.records.len(), 3);
        assert!(Arc::ptr_eq(&first, &state.records[0]));
        state.limits.max_total_work = work;
        assert_eq!(
            error_code(state.advance(&CancellationToken::default())),
            "authoring_trial_work_limit"
        );
        state.limits.execution.max_work = u64::MAX;
        assert_eq!(
            error_code(state.advance(&CancellationToken::default())),
            "authoring_limits_invalid"
        );
        assert_eq!(state.accepted_work, work);
    }

    #[test]
    fn d07_b09_b14_amount_query_states_are_typed_and_do_not_advance_the_trial() {
        use crate::authoring::{AmountBattle, AmountQuery};

        let token = ValidatedGameData::load_live_embedded().unwrap();
        let document = preparation_document(&token);
        let cancellation = CancellationToken::default();
        let disabled = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancellation,
        )
        .unwrap();
        let limits = ExecutionLimits::default();
        let result = disabled
            .preview_amount(
                AmountQuery::Latest {
                    battle: AmountBattle::Active,
                },
                &limits,
                &cancellation,
            )
            .unwrap();
        let value: Value = serde_json::from_str(result.document()).unwrap();
        assert_eq!(value["status"], "unavailable");
        assert_eq!(value["reason"], "observation_disabled");

        let enabled = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits {
                amount_observation: true,
                ..Default::default()
            },
            &cancellation,
        )
        .unwrap();
        let receipts = enabled.receipt_documents().collect::<Vec<_>>();
        let accepted_work = enabled.accepted_work();
        let next = enabled.next_step_index();
        let first = enabled
            .preview_amount(
                AmountQuery::Latest {
                    battle: AmountBattle::Active,
                },
                &limits,
                &cancellation,
            )
            .unwrap();
        let second = enabled
            .preview_amount(
                AmountQuery::Latest {
                    battle: AmountBattle::Active,
                },
                &limits,
                &cancellation,
            )
            .unwrap();
        assert_eq!(first.document(), second.document());
        let value: Value = serde_json::from_str(first.document()).unwrap();
        assert_eq!(value["status"], "unavailable");
        assert_eq!(value["reason"], "not_yet_evaluated");
        let terminal = enabled
            .explain_amount(
                AmountQuery::Latest {
                    battle: AmountBattle::LastTerminal,
                },
                &limits,
                &cancellation,
            )
            .unwrap();
        let terminal: Value = serde_json::from_str(terminal.document()).unwrap();
        assert_eq!(terminal["reason"], "no_battle");
        assert_eq!(enabled.receipt_documents().collect::<Vec<_>>(), receipts);
        assert_eq!(enabled.accepted_work(), accepted_work);
        assert_eq!(enabled.next_step_index(), next);
    }

    fn public_amount_trial_fixture(
        configure: impl FnOnce(&ExternalFixture),
    ) -> (ExternalFixture, ValidatedGameData, Value) {
        let fixture = ExternalFixture::official_with_legacy_starters();
        let fragments_path = "official/data/skill_fragments/base.ron";
        let fragments = fixture.read(fragments_path).replacen(
            "fragments: [",
            r#"fragments: [
SkillFragmentMetadata(
id:"authoring_preview_fragment",uuid:"00000000-0000-0000-0000-00000000ca05",
name:"Prototype manual preview",description:"Isolated query acceptance fixture",rarity:Rare,
equip_limit:OwnedCopies,origin:Some(Concept(concept_id:"authoring_preview")),sources:[RareReward],
dependencies:[],compatibility:(),effect:ActiveSkill(forms:{"base":(skill_id:"fragment_freischutz_black_round", resonance_max: 25)},dismantle_recipe:None)),"#,
            1,
        );
        fixture.write(fragments_path, fragments);
        let rewards_path = "official/data/events/rewards/base.ron";
        let rewards = fixture.read(rewards_path).replacen(
            "GrantEnkephalin(amount: 100),",
            "GrantEnkephalin(amount: 100),\n                GrantSkillFragment(fragment_id: \"authoring_preview_fragment\"),",
            1,
        );
        fixture.write(rewards_path, rewards);
        let enemies_path = "official/data/enemies/corroded_employees.ron";
        let enemies = fixture
            .read(enemies_path)
            .replacen("max_health: 70,", "max_health: 170,", 1)
            .replacen("attack: 12,", "attack: 1,", 1);
        fixture.write(enemies_path, enemies);
        configure(&fixture);
        let token = fixture.load();
        let mut document = preparation_document(&token);
        let cancellation = CancellationToken::default();
        let probe = TrialSession::prepare(
            fixture_content(&token),
            checked(&document),
            TrialLimits {
                amount_observation: true,
                ..Default::default()
            },
            &cancellation,
        )
        .unwrap();
        let zones = probe.step(4).unwrap().public_result().unwrap()["payload"]["combat_preview"]
            ["deployment_zones"]
            .as_array()
            .unwrap();
        let ground = zones
            .iter()
            .position(|zone| zone["kind"] == "Ground")
            .unwrap();
        let mut first_deploy = authority_step("deploy_unit", 6);
        first_deploy["command"]["employee_uuid"] = Value::Null;
        first_deploy["command"]["position"] = Value::Null;
        first_deploy["command"]["facing"] = json!("right");
        first_deploy["bindings"].as_array_mut().unwrap().extend([
            json!({"from":{"step":6,"pointer":"/payload/battle_update/checkpoint/deployment/unit_deploy_costs/0/employee_uuid"},
                "to":"/command/employee_uuid", "value_type":"string"}),
            json!({"from":{"step":4,"pointer":format!("/payload/combat_preview/deployment_zones/{ground}/cells/3")},
                "to":"/command/position", "value_type":"object"}),
        ]);
        let mut second_initial_deploy = authority_step("deploy_unit", 9);
        second_initial_deploy["command"]["employee_uuid"] = Value::Null;
        second_initial_deploy["command"]["position"] = Value::Null;
        second_initial_deploy["command"]["facing"] = json!("right");
        second_initial_deploy["bindings"]
            .as_array_mut()
            .unwrap()
            .extend([
                json!({"from":{"step":6,"pointer":"/payload/battle_update/checkpoint/deployment/unit_deploy_costs/1/employee_uuid"},
                    "to":"/command/employee_uuid", "value_type":"string"}),
                json!({"from":{"step":4,"pointer":format!("/payload/combat_preview/deployment_zones/{ground}/cells/2")},
                    "to":"/command/position", "value_type":"object"}),
            ]);
        drop(probe);

        let opening_steps = vec![
            first_deploy,
            second_initial_deploy,
            json!({"kind":"clock", "clock":{"mode":"elapsed", "delta_ms":8000, "after_seq":0}}),
            json!({"kind":"clock", "clock":{"mode":"elapsed", "delta_ms":60000, "after_seq":0}}),
            json!({"kind":"command", "command":{"type":"complete_combat_result"}}),
        ];
        document["steps"] = Value::Array(opening_steps.clone());
        let completion_discovery = simulate(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancellation,
        )
        .unwrap();
        let next_nodes = completion_discovery
            .step(13)
            .unwrap()
            .public_result()
            .unwrap()["payload"]["completion"]["payload"]["map"]["nodes"]
            .as_array()
            .unwrap();
        let reward_node_index = next_nodes
            .iter()
            .position(|node| node["state"] == "Available" && node["category"] == "Reward")
            .unwrap_or_else(|| panic!("no next public reward node: {next_nodes:?}"));

        let mut equip = json!({
            "kind":"command",
            "command":{"type":"equip_skill_fragment", "employee_uuid":Value::Null,
                "copy_uuid":Value::Null},
            "bindings":[{"from":{"step":13,"pointer":"/payload/skill_fragment_diffs/0/copy_uuid"}, "to":"/command/copy_uuid","value_type":"string"}]
        });
        equip["bindings"]
            .as_array_mut()
            .unwrap()
            .push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},
                "to":"/command/employee_uuid", "value_type":"string"}));
        let mut select_reward = json!({
            "kind":"command", "command":{"type":"select_map_node", "node_id":Value::Null},
            "bindings":[]
        });
        select_reward["bindings"].as_array_mut().unwrap().push(
            json!({"from":{"step":13,"pointer":format!("/payload/completion/payload/map/nodes/{reward_node_index}/id")},
                "to":"/command/node_id", "value_type":"string"}),
        );
        let mut bridge_steps = opening_steps;
        bridge_steps.extend([
            equip,
            select_reward,
            json!({"kind":"command", "command":{"type":"confirm_enter_node"}}),
            json!({"kind":"command", "command":{"type":"claim_reward"}}),
            json!({"kind":"command", "command":{"type":"exit_reward"}}),
        ]);
        document["steps"] = Value::Array(bridge_steps.clone());
        let reward_discovery = simulate(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancellation,
        )
        .unwrap();
        let reward_result = reward_discovery.step(18).unwrap().public_result().unwrap();
        let post_reward_nodes = reward_result["payload"]["map"]["nodes"]
            .as_array()
            .unwrap_or_else(|| panic!("unexpected public reward result: {reward_result}"));
        let combat_node_index = post_reward_nodes
            .iter()
            .position(|node| node["state"] == "Available" && node["category"] == "Combat")
            .unwrap_or_else(|| panic!("no combat after public reward: {post_reward_nodes:?}"));
        let mut select_combat = json!({
            "kind":"command", "command":{"type":"select_map_node", "node_id":Value::Null},
            "bindings":[]
        });
        select_combat["bindings"].as_array_mut().unwrap().push(
            json!({"from":{"step":18,"pointer":format!("/payload/map/nodes/{combat_node_index}/id")},
                "to":"/command/node_id", "value_type":"string"}),
        );
        let mut second_deploy = authority_step("deploy_unit", 21);
        second_deploy["command"]["employee_uuid"] = Value::Null;
        second_deploy["command"]["position"] = Value::Null;
        second_deploy["command"]["facing"] = json!("right");
        second_deploy["bindings"].as_array_mut().unwrap().extend([
            json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},
                "to":"/command/employee_uuid", "value_type":"string"}),
            json!({"from":{"step":19,"pointer":format!("/payload/combat_preview/deployment_zones/{ground}/cells/3")},
                "to":"/command/position", "value_type":"object"}),
        ]);
        let mut common_steps = bridge_steps;
        common_steps.extend([
            select_combat,
            json!({"kind":"command", "command":{"type":"confirm_enter_node"}}),
            authority_step("pause_battle", 20),
            json!({"kind":"snapshot", "snapshot":"run"}),
            second_deploy,
            json!({"kind":"clock", "clock":{"mode":"elapsed", "delta_ms":23000, "after_seq":0}}),
        ]);
        document["steps"] = Value::Array(common_steps.clone());
        let discovery = simulate(
            fixture_content(&token),
            checked(&document),
            TrialLimits::default(),
            &cancellation,
        )
        .unwrap();
        let checkpoint = &discovery.step(24).unwrap().public_result().unwrap()["payload"]
            ["battle_update"]["checkpoint"];
        let skill_id = checkpoint["deployment"]["deployed_units"][0]["skill_readiness"]["skill_id"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(skill_id, "fragment_freischutz_black_round");
        let enemy_index = checkpoint["units"]
            .as_array()
            .unwrap()
            .iter()
            .position(|unit| unit["owner"] == "Opponent")
            .unwrap();

        let mut begin = authority_step("begin_skill_targeting", 24);
        begin["command"]["employee_uuid"] = Value::Null;
        begin["command"]["skill_id"] = json!(skill_id);
        begin["bindings"]
            .as_array_mut()
            .unwrap()
            .push(json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},
                "to":"/command/employee_uuid", "value_type":"string"}));
        let mut activate = authority_step("activate_skill", 25);
        activate["command"]["employee_uuid"] = Value::Null;
        activate["command"]["skill_id"] = json!("fragment_freischutz_black_round");
        activate["command"]["target"] = json!({"type":"Unit", "unit_instance_id": Value::Null});
        activate["bindings"].as_array_mut().unwrap().extend([
            json!({"from":{"step":9,"pointer":"/payload/employee_uuid"},
                "to":"/command/employee_uuid", "value_type":"string"}),
            json!({"from":{"step":24,"pointer":format!("/payload/battle_update/checkpoint/units/{enemy_index}/unit_instance_id")},
                "to":"/command/target/unit_instance_id", "value_type":"string"}),
        ]);
        let mut steps = common_steps;
        steps.extend([
            begin,
            activate,
            json!({"kind":"clock", "clock":{"mode":"elapsed", "delta_ms":700, "after_seq":0}}),
        ]);
        document["steps"] = Value::Array(steps);
        (fixture, token, document)
    }

    #[test]
    fn d07_b15_public_trial_amount_preview_explain_and_exact_use_actual_legacy_execution() {
        use crate::authoring::{AmountBattle, AmountQuery, BattleKey};
        let (_fixture, token, document) = public_amount_trial_fixture(|_| {});
        let cancellation = CancellationToken::default();
        let session = simulate(
            fixture_content(&token),
            checked(&document),
            TrialLimits {
                amount_observation: true,
                ..Default::default()
            },
            &cancellation,
        )
        .unwrap();
        let limits = ExecutionLimits::default();
        let query = AmountQuery::Latest {
            battle: AmountBattle::LastTerminal,
        };
        let before = session.receipt_documents().collect::<Vec<_>>();
        let work = session.accepted_work();
        let preview = session
            .preview_amount(query, &limits, &cancellation)
            .unwrap();
        let explanation = session
            .explain_amount(query, &limits, &cancellation)
            .unwrap();
        let preview_json: Value = serde_json::from_str(preview.document()).unwrap();
        let explanation_json: Value = serde_json::from_str(explanation.document()).unwrap();
        assert_eq!(preview_json["status"], "evaluated", "{preview_json}");
        assert_eq!(preview_json["basis"], "recorded_evaluation");
        assert_eq!(
            preview_json["typed_base_amount"],
            explanation_json["typed_base_amount"]
        );
        assert_eq!(preview_json["trace"], explanation_json["trace"]);
        assert_eq!(
            explanation_json["static_plan"]["evaluation"]["performed"],
            false
        );
        let lifecycle_key = session
            .observation()
            .unwrap()
            .last_terminal()
            .unwrap()
            .key();
        let exact = AmountQuery::Exact {
            battle_key: BattleKey {
                entry_ordinal: lifecycle_key.entry_ordinal,
                battle_uuid: lifecycle_key.battle_uuid,
            },
            evaluation_ordinal: preview_json["identity"]["evaluation_ordinal"]
                .as_u64()
                .unwrap(),
        };
        let exact_document = session
            .preview_amount(exact, &limits, &cancellation)
            .unwrap();
        let exact_json: Value = serde_json::from_str(exact_document.document()).unwrap();
        assert_eq!(
            exact_json["typed_base_amount"],
            preview_json["typed_base_amount"]
        );
        let stale = session
            .preview_amount(
                AmountQuery::Exact {
                    battle_key: lifecycle_key,
                    evaluation_ordinal: exact_json["identity"]["evaluation_ordinal"]
                        .as_u64()
                        .unwrap()
                        + 1,
                },
                &limits,
                &cancellation,
            )
            .unwrap();
        let stale: Value = serde_json::from_str(stale.document()).unwrap();
        assert_eq!(stale["reason"], "not_retained");
        assert_eq!(session.receipt_documents().collect::<Vec<_>>(), before);
        assert_eq!(session.accepted_work(), work);
    }
}
