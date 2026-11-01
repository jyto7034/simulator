//! Read-only admission before any save-directory, lock, repair or profile load.
use super::active_run_store::{
    ActiveRunRecord, ActiveRunStore, ActiveRunStoreError, StoredCheckpoint,
};
use super::player_profile_store::PlayerProfileStore;
use game_core::{
    application::CoreApplication,
    mod_validation::{ApprovedHost, RequiredConfiguration},
};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub(crate) const PROFILE_MAX_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const ACTIVE_MAX_BYTES: usize = 96 * 1024 * 1024;
pub(crate) const JOURNAL_MAX_BYTES: usize = 96 * 1024 * 1024;

pub(crate) struct ApprovedStoreGate {
    pub(crate) active_root: PathBuf,
    pub(crate) profile_root: PathBuf,
    host: Arc<ApprovedHost>,
    pinned: Mutex<RequiredConfiguration>,
}
impl ApprovedStoreGate {
    pub(crate) fn new(
        active_root: PathBuf,
        profile_root: PathBuf,
        host: Arc<ApprovedHost>,
    ) -> Result<Arc<Self>, ActiveRunStoreError> {
        let (_, pinned) = host.future_generation().map_err(protected)?;
        let gate = Arc::new(Self {
            active_root,
            profile_root,
            host,
            pinned: Mutex::new(pinned),
        });
        gate.preflight()?;
        Ok(gate)
    }
    fn allowed(&self) -> Result<Vec<RequiredConfiguration>, ActiveRunStoreError> {
        let (_, future) = self.host.future_generation().map_err(protected)?;
        let pinned = self
            .pinned
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let mut allowed = vec![pinned.clone()];
        if future != pinned {
            allowed.push(future);
        }
        Ok(allowed)
    }
    pub(crate) fn preflight(&self) -> Result<(), ActiveRunStoreError> {
        let allowed = self.allowed()?;
        PlayerProfileStore::preflight_approved(&self.profile_root, &self.active_root, &allowed)
            .map_err(|e| ActiveRunStoreError::ProtectedOriginal(e.to_string()))
    }
    pub(crate) fn with_acceptance<T, E>(
        &self,
        required: &RequiredConfiguration,
        require_future: bool,
        accept: impl FnOnce() -> Result<T, E>,
    ) -> Result<Result<T, E>, ActiveRunStoreError> {
        self.host
            .with_installed_selection(|selected| {
                if require_future && &selected.required != required {
                    return Err(game_core::mod_validation::ModValidationError::new(
                        game_core::mod_validation::ModValidationErrorKind::Stale,
                        "save.accept",
                        "future selection changed after preparation",
                    ));
                }
                self.ensure_allowed(required).map_err(|e| {
                    game_core::mod_validation::ModValidationError::new(
                        game_core::mod_validation::ModValidationErrorKind::ProtectedOriginal,
                        "save.accept",
                        e.to_string(),
                    )
                })?;
                let result = accept();
                if result.is_ok() {
                    // The callback reports Ok only after the journal rename was
                    // accepted. One current durable configuration, no history list.
                    // Assignment is atomic under this mutex; a prior poison cannot
                    // turn an already accepted journal into a preaccept refusal.
                    *self
                        .pinned
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) = required.clone();
                }
                Ok(result)
            })
            .map_err(protected)
    }
    pub(crate) fn ensure_allowed(
        &self,
        required: &RequiredConfiguration,
    ) -> Result<(), ActiveRunStoreError> {
        required.validate().map_err(protected)?;
        if !self.allowed()?.contains(required) {
            return Err(protected(
                "configuration was not installed by this approved host",
            ));
        }
        Ok(())
    }
    pub(crate) fn configuration_for_checkpoint(
        &self,
        checkpoint: Option<&StoredCheckpoint>,
        previous: Option<&RequiredConfiguration>,
    ) -> Result<RequiredConfiguration, ActiveRunStoreError> {
        let allowed = self.allowed()?;
        if let Some(checkpoint) = checkpoint {
            let identity = CoreApplication::inspect_checkpoint_requirements(
                &checkpoint.core_payload_json,
                64 * 1024 * 1024,
            )
            .map_err(protected)?;
            if let Some(previous) =
                previous.filter(|p| p.content_identity == identity && allowed.contains(p))
            {
                return Ok(previous.clone());
            }
            let candidates: Vec<_> = allowed
                .into_iter()
                .filter(|p| p.content_identity == identity)
                .collect();
            if candidates.len() != 1 {
                return Err(protected(
                    "ambiguous or unapproved checkpoint configuration",
                ));
            }
            return Ok(candidates.into_iter().next().unwrap());
        }
        if let Some(previous) = previous {
            self.ensure_allowed(previous)?;
            return Ok(previous.clone());
        }
        Ok(self
            .pinned
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone())
    }
}
pub(crate) fn protected(reason: impl std::fmt::Display) -> ActiveRunStoreError {
    ActiveRunStoreError::ProtectedOriginal(reason.to_string())
}
pub(crate) fn read_optional(
    path: &Path,
    limit: u64,
) -> Result<Option<Vec<u8>>, ActiveRunStoreError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if !metadata.is_file() || metadata.len() > limit {
        return Err(protected(format!(
            "{}: unsupported original file type/bytes",
            path.display()
        )));
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(protected("original changed file type"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(protected("original exceeds byte ceiling"));
    }
    Ok(Some(bytes))
}
pub(crate) fn check_record(
    record: &ActiveRunRecord,
    allowed: &[RequiredConfiguration],
) -> Result<(), ActiveRunStoreError> {
    if let Some(required) = &record.required_configuration {
        required.validate().map_err(protected)?;
        if !allowed.contains(required) {
            return Err(protected(
                "required mods/packs/writers differ from approved host",
            ));
        }
        if let Some(checkpoint) = &record.checkpoint {
            let identity = CoreApplication::inspect_checkpoint_requirements(
                &checkpoint.core_payload_json,
                64 * 1024 * 1024,
            )
            .map_err(protected)?;
            if identity != required.content_identity {
                return Err(protected("Core/required configuration identity differs"));
            }
        }
    } else if let Some(checkpoint) = &record.checkpoint {
        let identity = CoreApplication::inspect_checkpoint_requirements(
            &checkpoint.core_payload_json,
            64 * 1024 * 1024,
        )
        .map_err(protected)?;
        if !allowed.iter().any(|a| a.content_identity == identity) {
            return Err(protected(
                "legacy full content identity differs from approved host",
            ));
        }
    }
    Ok(())
}
pub(crate) fn inspect_active(
    active_root: &Path,
    allowed: &[RequiredConfiguration],
    accepted: Option<(&ActiveRunRecord, &ActiveRunRecord)>,
) -> Result<(), ActiveRunStoreError> {
    let copies = ActiveRunStore::read_originals(active_root)?;
    let mut valid = 0;
    for (_, copy) in &copies {
        match copy {
            Ok(record) => {
                if let Some((old, new)) = accepted {
                    if record != old && record != new {
                        return Err(protected("valid original is neither exact accepted old record nor installed new record"));
                    }
                } else {
                    check_record(record, allowed)?;
                }
                valid += 1;
            }
            Err(
                error @ (ActiveRunStoreError::UnsupportedVersion(_)
                | ActiveRunStoreError::UnsupportedCoreVersion(_)
                | ActiveRunStoreError::ProtectedOriginal(_)
                | ActiveRunStoreError::Io(_)),
            ) => return Err(protected(error)),
            Err(_) => {}
        }
    }
    if !copies.is_empty() && valid == 0 {
        return Err(protected(
            "both original active-run copies are invalid or identity is uncertain",
        ));
    }
    if let Some((old, _)) = accepted {
        if valid == 0 && !old.revision.is_nil() {
            return Err(protected("accepted old original is missing"));
        }
    }
    Ok(())
}
