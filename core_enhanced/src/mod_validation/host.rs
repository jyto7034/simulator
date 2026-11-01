use super::{ApprovedGeneration, ModValidationError, RequiredConfiguration, ValidationIssuer};
use crate::{
    application::{
        ApplicationCommand, ApplicationCommandKind, ApplicationError, ApplicationExecutionContext,
        CoreApplication, PreparedNonBattleApplication,
    },
    game::data::ValidatedGameData,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};
#[derive(Debug, Clone, Serialize)]
pub struct HostSelectionView {
    pub coverage_support_revision: &'static str,
    pub candidate_identity: String,
    pub selection_revision: u64,
    pub required: RequiredConfiguration,
}
#[derive(Debug, Serialize)]
pub enum HostApplicationError {
    Admission(ModValidationError),
    Application(ApplicationError),
}
pub struct ApprovedHost {
    issuer: Arc<ValidationIssuer>,
    future: Mutex<ApprovedGeneration>,
    installation: Mutex<()>,
}
impl ApprovedHost {
    pub fn new(
        issuer: Arc<ValidationIssuer>,
        initial: ApprovedGeneration,
    ) -> Result<Arc<Self>, ModValidationError> {
        issuer.with_current_approval(&initial, || {
            Ok(Arc::new(Self {
                issuer: Arc::clone(&issuer),
                future: Mutex::new(initial.clone()),
                installation: Mutex::new(()),
            }))
        })
    }
    pub fn apply(
        &self,
        approved: ApprovedGeneration,
    ) -> Result<HostSelectionView, ModValidationError> {
        // The issuer lock and future-selection replacement must be one serial
        // operation; a concurrent invalidate cannot slip between check/install.
        self.issuer.with_current_approval(&approved, || {
            let _installation = self.installation.lock().map_err(|_| {
                ModValidationError::incomplete("host.installation", "poisoned installation owner")
            })?;
            let mut future = self.future.lock().map_err(|_| {
                ModValidationError::incomplete("host.future", "poisoned future owner")
            })?;
            *future = approved.clone();
            Ok(view(&future))
        })
    }
    pub fn query(&self) -> Result<HostSelectionView, ModValidationError> {
        self.future
            .lock()
            .map(|a| view(&a))
            .map_err(|_| ModValidationError::incomplete("host.future", "poisoned future owner"))
    }
    /// Serializes a normal consumer's acceptance with host Apply. The installed
    /// selection remains valid while the issuer is validating a later request.
    pub fn with_installed_selection<T>(
        &self,
        accept: impl FnOnce(&HostSelectionView) -> Result<T, ModValidationError>,
    ) -> Result<T, ModValidationError> {
        let _installation = self.installation.lock().map_err(|_| {
            ModValidationError::incomplete("host.installation", "poisoned installation owner")
        })?;
        let selected = self.query()?;
        accept(&selected)
    }
    pub fn future_generation(
        &self,
    ) -> Result<(ValidatedGameData, RequiredConfiguration), ModValidationError> {
        let future = self
            .future
            .lock()
            .map_err(|_| ModValidationError::incomplete("host.future", "poisoned future owner"))?;
        Ok((
            future.generation().clone(),
            future.required_configuration().clone(),
        ))
    }
    pub fn new_application(
        &self,
        seed: u64,
    ) -> Result<(CoreApplication, RequiredConfiguration), ModValidationError> {
        let (generation, required) = self.future_generation()?;
        Ok((CoreApplication::new(generation, seed), required))
    }
    pub fn prepare_with_future_generation<'a>(
        &self,
        application: &'a mut CoreApplication,
        active_required: &RequiredConfiguration,
        player: uuid::Uuid,
        command: ApplicationCommand,
        context: ApplicationExecutionContext,
    ) -> Result<(PreparedNonBattleApplication<'a>, RequiredConfiguration), HostApplicationError>
    {
        active_required
            .validate()
            .map_err(HostApplicationError::Admission)?;
        if &active_required.content_identity != application.content_identity() {
            return Err(HostApplicationError::Admission(
                ModValidationError::protected("active application requirement identity mismatch"),
            ));
        }
        let future = self.future.lock().map_err(|_| {
            HostApplicationError::Admission(ModValidationError::incomplete(
                "host.future",
                "poisoned future owner",
            ))
        })?;
        let starts_new_game = command.kind() == ApplicationCommandKind::StartNewGame;
        let required = if starts_new_game {
            future.required_configuration().clone()
        } else {
            active_required.clone()
        };
        let generation = starts_new_game.then(|| future.generation().clone());
        let prepared = application
            .prepare_command_with_generation(
                player,
                command,
                context,
                generation,
            )
            .map_err(HostApplicationError::Application)?;
        Ok((prepared, required))
    }
}
fn view(a: &ApprovedGeneration) -> HostSelectionView {
    HostSelectionView {
        coverage_support_revision: super::COVERAGE_SUPPORT_REVISION,
        candidate_identity: a.candidate_identity().into(),
        selection_revision: a.selection_revision(),
        required: a.required_configuration().clone(),
    }
}
