//! Normal host persistence. Its constructors bind only installed host selections;
//! arbitrary DTO metadata cannot authorize a save or writer-capable recovery.
use crate::persistence::{
    active_run_store::{ActiveRunSnapshot, ActiveRunStore},
    mod_requirements::ApprovedStoreGate,
    player_profile_store::{PlayerProfilePersistence, PlayerProfileStore},
};
use game_core::{
    application::{ApplicationCheckpoint, PreparedApplication},
    mod_validation::{ApprovedHost, RequiredConfiguration},
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub struct ApprovedPersistence {
    pub(crate) active: Arc<Mutex<ActiveRunStore>>,
    pub(crate) profile: Arc<Mutex<PlayerProfilePersistence>>,
}
impl ApprovedPersistence {
    pub fn open(root: impl AsRef<Path>, host: Arc<ApprovedHost>) -> Result<Self, String> {
        Self::open_roots(
            root.as_ref().to_path_buf(),
            root.as_ref().to_path_buf(),
            host,
        )
    }
    pub fn open_roots(
        active_root: PathBuf,
        profile_root: PathBuf,
        host: Arc<ApprovedHost>,
    ) -> Result<Self, String> {
        // Read-only inspection precedes both store constructors and bootstrap.
        let gate = ApprovedStoreGate::new(active_root.clone(), profile_root.clone(), host)
            .map_err(|e| e.to_string())?;
        let active = ActiveRunStore::open_approved(active_root, Arc::clone(&gate))
            .map_err(|e| e.to_string())?;
        let store =
            PlayerProfileStore::open_approved(profile_root, gate).map_err(|e| e.to_string())?;
        let mut profile = PlayerProfilePersistence::bootstrap(store);
        profile
            .recover_pending_application(&active)
            .map_err(|e| e.to_string())?;
        profile.profile().map_err(str::to_owned)?;
        Ok(Self {
            active: Arc::new(Mutex::new(active)),
            profile: Arc::new(Mutex::new(profile)),
        })
    }
    /// Capture the exact profile and its active revision before preparing a command.
    pub fn preparation_snapshot(&self) -> Result<(game_core::game::encounter_compendium::PlayerProfileState, ActiveRunSnapshot), String> {
        let profile = self.profile.lock().map_err(|_| "profile store lock poisoned".to_owned())?;
        let active = self.active.lock().map_err(|_| "active store lock poisoned".to_owned())?;
        Ok((profile.profile().map_err(str::to_owned)?, active.snapshot().map_err(|e| e.to_string())?))
    }

    pub fn snapshot(&self) -> Result<ActiveRunSnapshot, String> {
        self.active
            .lock()
            .map_err(|_| "active store lock poisoned".to_owned())?
            .snapshot()
            .map_err(|e| e.to_string())
    }
    pub fn player_profile(
        &self,
    ) -> Result<game_core::game::encounter_compendium::PlayerProfileState, String> {
        self.profile
            .lock()
            .map_err(|_| "profile store lock poisoned".to_owned())?
            .profile()
            .map_err(str::to_owned)
    }
    pub fn publish<T>(
        &self,
        prepared: PreparedApplication<'_, T>,
        required: &RequiredConfiguration,
        expected_revision: &mut Option<uuid::Uuid>,
        checkpoint_before: Option<&ApplicationCheckpoint>,
        force_revision: bool,
    ) -> Result<(T, bool), String> {
        let mut profile = self
            .profile
            .lock()
            .map_err(|_| "profile store lock poisoned".to_owned())?;
        let current = profile.profile().map_err(str::to_owned)?;
        let mut merged = prepared.player_profile().clone();
        merged.validate_persisted().map_err(|e| e.to_string())?;
        merged.merge_knowledge(&current);
        let active = self.active.lock().map_err(|_| "active store lock poisoned".to_owned())?;
        let expected = expected_revision.unwrap_or(uuid::Uuid::nil());
        let actual = active.snapshot().map_err(|e| e.to_string())?.revision;
        if expected != actual { return Err(format!("active revision conflict: expected {expected}, actual {actual}")); }
        if !force_revision && !prepared.terminal() && prepared.checkpoint() == checkpoint_before && merged == current {
            return Ok((prepared.commit(), false));
        }
        let checkpoint = prepared
            .checkpoint()
            .map(crate::persistence::active_run_store::StoredCheckpoint::from);
        let accepted = profile
            .accept_application_with_requirement(
                &active,
                expected,
                checkpoint.as_ref(),
                &merged,
                Some(required),
                force_revision,
            )
            .map_err(|e| e.to_string())?;
        *expected_revision = Some(accepted.revision);
        // U04: acceptance consumes this Core draft exactly once, including when
        // filesystem completion must be retried after the commit.
        Ok((prepared.commit(), accepted.recovery_pending))
    }
}
