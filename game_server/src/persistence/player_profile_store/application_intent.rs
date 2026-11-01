use super::*;
use crate::persistence::active_run_store::{
    ActiveRunRecord, ActiveRunRevision, ActiveRunStore, StoredCheckpoint, CorruptActiveOriginals,
};

const JOURNAL_VERSION: u32 = 4;
const JOURNAL_FILE: &str = "application_commit.pending.json";
const JOURNAL_TEMP_FILE: &str = "application_commit.pending.tmp";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplicationIntent {
    version: u32,
    expected_revision: ActiveRunRevision,
    active: ActiveRunRecord,
    profile: PlayerProfileState,
    #[serde(deserialize_with = "required_intent_option")]
    corrupt_active_originals: Option<CorruptActiveOriginals>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    old_active: Option<ActiveRunRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    old_required_configuration: Option<game_core::mod_validation::RequiredConfiguration>,
}
fn required_intent_option<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where D: serde::Deserializer<'de>, T: Deserialize<'de> { Option::<T>::deserialize(d) }

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyActiveRecord { revision: ActiveRunRevision, checkpoint: Option<StoredCheckpoint> }
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplicationIntentV1 { version: u32, expected_revision: ActiveRunRevision, active: LegacyActiveRecord, profile: PlayerProfileState }
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntentEnvelopeV1 { intent: ApplicationIntentV1, checksum_sha256: String }
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntentEnvelope {
    intent: ApplicationIntent,
    checksum_sha256: String,
}

/// Once accepted, the caller must commit its prepared Core exactly once even
/// when finishing the filesystem publications requires recovery.
#[derive(Debug, Clone)]
pub struct AcceptedApplication {
    pub revision: ActiveRunRevision,
    pub recovery_pending: bool,
}

impl IntentEnvelope {
    fn encode(intent: &ApplicationIntent) -> Result<Vec<u8>, PlayerProfileStoreError> {
        intent
            .profile
            .validate_persisted()
            .map_err(PlayerProfileStoreError::InvalidEnvelope)?;
        PlayerProfileEnvelope::encode(&intent.profile)?;
        ActiveRunStore::validate_record(&intent.active)
            .map_err(PlayerProfileStoreError::ActiveRun)?;
        if intent.version != JOURNAL_VERSION || intent.old_active.as_ref().is_none_or(|old| old.revision != intent.expected_revision)
            || (intent.active.required_configuration.is_some() && intent.old_required_configuration.is_none()) {
            return Err(PlayerProfileStoreError::ProtectedOriginal("incoherent exact lifecycle journal".into()));
        }
        let bytes = serde_json::to_vec(intent)
            .map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        let bytes = serde_json::to_vec_pretty(&Self {
            intent: intent.clone(), checksum_sha256: format!("{:x}", Sha256::digest(bytes)),
        }).map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        if bytes.len() > super::super::mod_requirements::JOURNAL_MAX_BYTES { return Err(PlayerProfileStoreError::InvalidEnvelope("accepted journal exceeds supported byte ceiling".into())); }
        Ok(bytes)
    }
    fn decode(bytes: &[u8]) -> Result<ApplicationIntent, PlayerProfileStoreError> {
        if bytes.len() > super::super::mod_requirements::JOURNAL_MAX_BYTES { return Err(PlayerProfileStoreError::InvalidEnvelope("accepted journal exceeds supported byte ceiling".into())); }
        // Inspect the version before interpreting the version-specific shape.
        #[derive(Deserialize)]
        struct Header {
            intent: Version,
        }
        #[derive(Deserialize)]
        struct Version {
            version: u32,
        }
        let header: Header = serde_json::from_slice(bytes)
            .map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        if header.intent.version != JOURNAL_VERSION {
            return Err(PlayerProfileStoreError::ProtectedOriginal(format!(
                "{JOURNAL_FILE} version {}",
                header.intent.version
            )));
        }
        let envelope: Self = serde_json::from_slice(bytes)
            .map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        let encoded = serde_json::to_vec(&envelope.intent)
            .map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        if envelope.checksum_sha256 != format!("{:x}", Sha256::digest(encoded)) {
            return Err(PlayerProfileStoreError::ChecksumMismatch);
        }
        validate_intent(&envelope.intent)?;
        if envelope.intent.old_active.as_ref().is_none_or(|old| old.revision != envelope.intent.expected_revision) || (envelope.intent.active.required_configuration.is_some() && envelope.intent.old_required_configuration.is_none()) {
            return Err(PlayerProfileStoreError::ProtectedOriginal("V4 journal lacks exact old/new configuration binding".into()));
        }
        if let Some(old) = &envelope.intent.old_active {
            if !old.revision.is_nil() { ActiveRunStore::validate_record(old).map_err(PlayerProfileStoreError::ActiveRun)?; }
        }
        Ok(envelope.intent)
    }
}
fn validate_intent(intent: &ApplicationIntent) -> Result<(), PlayerProfileStoreError> {
        if intent.corrupt_active_originals.is_some() && (!intent.expected_revision.is_nil()
            || intent.active.checkpoint.is_some() || intent.profile.lifecycle.active_expedition.is_some()
            || intent.old_active.as_ref().is_none_or(|old| !old.revision.is_nil() || old.checkpoint.is_some())) {
            return Err(PlayerProfileStoreError::InvalidEnvelope("invalid explicit corrupt-active reset intent".into()));
        }
        if intent.active.revision == intent.expected_revision {
            return Err(PlayerProfileStoreError::InvalidEnvelope(
                "intent must advance its active revision".into(),
            ));
        }
        intent
            .profile
            .validate_persisted()
            .map_err(PlayerProfileStoreError::InvalidEnvelope)?;
        ActiveRunStore::validate_record(&intent.active)
            .map_err(PlayerProfileStoreError::ActiveRun)?;
        Ok(())
}

impl PlayerProfileStore {
    pub(crate) fn preflight_approved(profile_root: &Path, active_root: &Path, allowed: &[game_core::mod_validation::RequiredConfiguration]) -> Result<(), PlayerProfileStoreError> {
        use crate::persistence::mod_requirements::{read_optional, inspect_active, check_record};
        for name in [LEGACY_CURRENT_FILE, LEGACY_BACKUP_FILE, LEGACY_TEMP_FILE, LEGACY_BACKUP_TEMP_FILE, STANDARD_COMPLETION_JOURNAL_FILE, STANDARD_COMPLETION_JOURNAL_TEMP_FILE] {
            if profile_root.join(name).exists() { return Err(PlayerProfileStoreError::ProtectedOriginal(name.into())); }
        }
        let mut profile_copies = 0;
        let mut valid_profile = 0;
        for name in [CURRENT_FILE, BACKUP_FILE] {
            if let Some(bytes) = read_optional(&profile_root.join(name), super::super::mod_requirements::PROFILE_MAX_BYTES as u64).map_err(PlayerProfileStoreError::ActiveRun)? {
                profile_copies += 1;
                match PlayerProfileEnvelope::decode(&bytes) {
                    Ok(_) => valid_profile += 1,
                    Err(error @ PlayerProfileStoreError::UnsupportedVersion(_)) => return Err(error),
                    Err(_) => {},
                }
            }
        }
        let intent = read_optional(&profile_root.join(JOURNAL_FILE), super::super::mod_requirements::JOURNAL_MAX_BYTES as u64).map_err(PlayerProfileStoreError::ActiveRun)?
            .map(|bytes| IntentEnvelope::decode(&bytes)).transpose()?;
        if profile_copies > 0 && valid_profile == 0 && intent.is_none() { return Err(PlayerProfileStoreError::ProtectedOriginal("both original profile copies invalid without an authoritative accepted intent".into())); }
        if let Some(intent) = intent {
            check_record(&intent.active, allowed).map_err(PlayerProfileStoreError::ActiveRun)?;
            if let Some(originals) = &intent.corrupt_active_originals {
                if let Some(required) = &intent.old_required_configuration {
                    if !allowed.contains(required) { return Err(PlayerProfileStoreError::ProtectedOriginal("reset old configuration is unapproved".into())); }
                }
                return ActiveRunStore::verify_corrupt_reset_originals(active_root, originals, &intent.active).map_err(PlayerProfileStoreError::ActiveRun);
            }
            if intent.old_required_configuration.is_some() {
                let old = intent.old_active.as_ref().ok_or_else(|| PlayerProfileStoreError::ProtectedOriginal("accepted old record missing".into()))?;
                let required = intent.old_required_configuration.as_ref().ok_or_else(|| PlayerProfileStoreError::ProtectedOriginal("accepted old required configuration missing".into()))?;
                required.validate().map_err(|e| PlayerProfileStoreError::ProtectedOriginal(e.to_string()))?;
                if old.required_configuration.as_ref().is_some_and(|r| r != required) { return Err(PlayerProfileStoreError::ProtectedOriginal("accepted old metadata differs".into())); }
                if let Some(checkpoint) = &old.checkpoint {
                    let identity = game_core::application::CoreApplication::inspect_checkpoint_requirements(&checkpoint.core_payload_json, 64 * 1024 * 1024).map_err(|e| PlayerProfileStoreError::ProtectedOriginal(e.to_string()))?;
                    if identity != required.content_identity { return Err(PlayerProfileStoreError::ProtectedOriginal("accepted old Core identity differs".into())); }
                }
                inspect_active(active_root, allowed, Some((old, &intent.active))).map_err(PlayerProfileStoreError::ActiveRun)
            } else {
                inspect_active(active_root, allowed, None).map_err(PlayerProfileStoreError::ActiveRun)
            }
        } else {
            inspect_active(active_root, allowed, None).map_err(PlayerProfileStoreError::ActiveRun)
        }
    }

    pub(super) fn protect_originals(&self) -> Result<(), PlayerProfileStoreError> {
        if let Some(gate) = &self.gate { gate.preflight().map_err(PlayerProfileStoreError::ActiveRun)?; }
        self.protect_profile_originals()
    }

    fn protect_profile_originals(&self) -> Result<(), PlayerProfileStoreError> {
        for file in [
            LEGACY_CURRENT_FILE,
            LEGACY_BACKUP_FILE,
            LEGACY_TEMP_FILE,
            LEGACY_BACKUP_TEMP_FILE,
            STANDARD_COMPLETION_JOURNAL_FILE,
            STANDARD_COMPLETION_JOURNAL_TEMP_FILE,
        ] {
            if self.path(file).exists() {
                return Err(PlayerProfileStoreError::ProtectedOriginal(file.into()));
            }
        }
        for file in [CURRENT_FILE, BACKUP_FILE] {
            match self.read_decoded(file) {
                Err(error @ PlayerProfileStoreError::UnsupportedVersion(_)) => return Err(error),
                Err(PlayerProfileStoreError::Io(error)) => return Err(error.into()),
                _ => {}
            }
        }
        // A pending journal is authoritative, including when malformed. Never
        // overwrite it, fall back past it, or clear it through a reset command.
        self.load_application_intent()?;
        Ok(())
    }

    pub(super) fn require_no_application_intent(&self) -> Result<(), PlayerProfileStoreError> {
        if self.load_application_intent()?.is_some() {
            return Err(PlayerProfileStoreError::InvalidEnvelope(
                "application intent requires recovery".into(),
            ));
        }
        Ok(())
    }

    fn load_application_intent(
        &self,
    ) -> Result<Option<ApplicationIntent>, PlayerProfileStoreError> {
        match fs::read(self.path(JOURNAL_FILE)) {
            Ok(bytes) => IntentEnvelope::decode(&bytes).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Err means no intent was installed. Once rename succeeds, even a directory
    /// sync error is an accepted/pending outcome: restart may observe the intent.
    fn stage_application_intent(&self, intent: &ApplicationIntent) -> Result<Option<String>, PlayerProfileStoreError> {
        self.stage_application_intent_checked(intent, || Ok(()))
    }

    fn stage_application_intent_checked(&self, intent: &ApplicationIntent, before_accept: impl FnOnce() -> Result<(), PlayerProfileStoreError>) -> Result<Option<String>, PlayerProfileStoreError> {
        if intent.corrupt_active_originals.is_some() { self.protect_profile_originals()?; } else { self.protect_originals()?; }
        self.require_no_application_intent()?;
        let bytes = IntentEnvelope::encode(intent)?;
        IntentEnvelope::decode(&bytes)?;
        self.write_synced(JOURNAL_TEMP_FILE, &bytes)?;
        crate::persistence::durable_crash_point("intent_temp_synced");
        before_accept()?;
        replace_file(&self.path(JOURNAL_TEMP_FILE), &self.path(JOURNAL_FILE))?;
        crate::persistence::durable_crash_point("intent_renamed");
        if let Err(error) = self.sync_root() {
            return Ok(Some(error.to_string()));
        }
        crate::persistence::durable_crash_point("intent_synced");
        Ok(None)
    }

    fn finish_application_intent(
        &self,
        active: &ActiveRunStore,
        intent: &ApplicationIntent,
    ) -> Result<(), PlayerProfileStoreError> {
        self.protect_originals()?;
        // Check both protected originals and the current revision before writing
        // either aggregate, including during startup recovery.
        if let Some(originals) = &intent.corrupt_active_originals {
            active.validate_corrupt_reset_install(originals, &intent.active).map_err(PlayerProfileStoreError::ActiveRun)?;
        } else {
            active.validate_install(intent.expected_revision, &intent.active).map_err(PlayerProfileStoreError::ActiveRun)?;
        }
        self.write_confirmed_profile(&intent.profile)?;
        if let Some(originals) = &intent.corrupt_active_originals {
            active.install_corrupt_reset(originals, &intent.active).map_err(PlayerProfileStoreError::ActiveRun)?;
        } else {
            active.install_record(intent.expected_revision, &intent.active).map_err(PlayerProfileStoreError::ActiveRun)?;
        }
        crate::persistence::durable_crash_point("before_intent_clear");
        fs::remove_file(self.path(JOURNAL_FILE))?;
        crate::persistence::durable_crash_point("intent_removed");
        self.sync_root()?;
        crate::persistence::durable_crash_point("intent_clear_synced");
        Ok(())
    }
}

impl PlayerProfilePersistence {
    pub fn accept_application(
        &mut self,
        active: &ActiveRunStore,
        expected_revision: ActiveRunRevision,
        checkpoint: Option<&StoredCheckpoint>,
        profile: &PlayerProfileState,
    ) -> Result<AcceptedApplication, PlayerProfileStoreError> {
        self.accept_application_with_requirement(active, expected_revision, checkpoint, profile, None, false)
    }

    pub fn accept_application_with_requirement(
        &mut self, active: &ActiveRunStore, expected_revision: ActiveRunRevision,
        checkpoint: Option<&StoredCheckpoint>, profile: &PlayerProfileState,
        required: Option<&game_core::mod_validation::RequiredConfiguration>, require_future: bool,
    ) -> Result<AcceptedApplication, PlayerProfileStoreError> {
        if let PlayerProfilePersistenceState::ApplicationRecoveryPending { issue }
        | PlayerProfilePersistenceState::Protected { issue }
        | PlayerProfilePersistenceState::Fatal { issue } = &self.state
        {
            return Err(PlayerProfileStoreError::InvalidEnvelope(issue.clone()));
        }
        self.store.protect_originals()?;
        self.store.require_no_application_intent()?;
        let old_active = active.readonly_record().map_err(PlayerProfileStoreError::ActiveRun)?;
        let mut record = active
            .prepare_record(expected_revision, checkpoint)
            .map_err(PlayerProfileStoreError::ActiveRun)?;
        if let Some(required) = required { active.bind_record(&mut record, required).map_err(PlayerProfileStoreError::ActiveRun)?; }
        let mut merged = profile.clone();
        merged.validate_persisted().map_err(PlayerProfileStoreError::InvalidEnvelope)?;
        merged.merge_knowledge(&self.profile);
        let intent = ApplicationIntent {
            version: JOURNAL_VERSION,
            expected_revision,
            active: record,
            profile: merged,
            corrupt_active_originals: None,
            old_required_configuration: self.store.gate.as_ref().map(|g| g.configuration_for_checkpoint(old_active.checkpoint.as_ref(), old_active.required_configuration.as_ref())).transpose().map_err(PlayerProfileStoreError::ActiveRun)?,
            old_active: Some(old_active),
        };
        let pending_issue = match (&self.store.gate, &intent.active.required_configuration) {
            (Some(gate), Some(required)) => gate.with_acceptance(required, require_future, || self.store.stage_application_intent(&intent)).map_err(PlayerProfileStoreError::ActiveRun)??,
            _ => self.store.stage_application_intent(&intent)?,
        };
        // There is now exactly one accepted candidate. Keep its exact lifecycle and knowledge
        // even if a later profile/active/backup publication fails.
        self.profile = intent.profile.clone();
        let failure = pending_issue.or_else(|| {
            self.store
                .finish_application_intent(active, &intent)
                .err()
                .map(|error| error.to_string())
        });
        self.recovery_notification_pending = false;
        self.state = match &failure {
            Some(issue) => PlayerProfilePersistenceState::ApplicationRecoveryPending {
                issue: issue.clone(),
            },
            None => PlayerProfilePersistenceState::Clean,
        };
        Ok(AcceptedApplication {
            revision: intent.active.revision,
            recovery_pending: failure.is_some(),
        })
    }

    /// Explicitly discard supported corrupt active copies and close only the current run marker.
    /// The caller holds profile -> active locks; profile-only reset cannot perform this operation.
    pub fn accept_corrupt_active_reset(&mut self, active: &ActiveRunStore) -> Result<AcceptedApplication, PlayerProfileStoreError> {
        if matches!(self.state, PlayerProfilePersistenceState::ApplicationRecoveryPending { .. } | PlayerProfilePersistenceState::Protected { .. } | PlayerProfilePersistenceState::Fatal { .. }) {
            return Err(PlayerProfileStoreError::InvalidEnvelope("resolve profile/accepted intent before corrupt active reset".into()));
        }
        self.store.protect_profile_originals()?;
        self.store.require_no_application_intent()?;
        let (originals, record) = active.prepare_corrupt_reset().map_err(PlayerProfileStoreError::ActiveRun)?;
        active.validate_corrupt_reset_install(&originals, &record).map_err(PlayerProfileStoreError::ActiveRun)?;
        let mut next_profile = self.profile.clone();
        next_profile.lifecycle.active_expedition = None;
        next_profile.validate_persisted().map_err(PlayerProfileStoreError::InvalidEnvelope)?;
        let intent = ApplicationIntent {
            version: JOURNAL_VERSION, expected_revision: uuid::Uuid::nil(),
            old_required_configuration: record.required_configuration.clone(),
            old_active: Some(ActiveRunRecord { revision: uuid::Uuid::nil(), checkpoint: None, required_configuration: record.required_configuration.clone() }),
            active: record, profile: next_profile, corrupt_active_originals: Some(originals),
        };
        // Recheck both exact originals immediately before the acceptance rename.
        let accept = || {
            active.validate_corrupt_reset_install(intent.corrupt_active_originals.as_ref().unwrap(), &intent.active).map_err(PlayerProfileStoreError::ActiveRun)?;
            self.store.stage_application_intent_checked(&intent, || active.validate_corrupt_reset_install(intent.corrupt_active_originals.as_ref().unwrap(), &intent.active).map_err(PlayerProfileStoreError::ActiveRun))
        };
        let pending = match (&self.store.gate, &intent.active.required_configuration) {
            (Some(gate), Some(required)) => gate.with_acceptance(required, false, accept).map_err(PlayerProfileStoreError::ActiveRun)??,
            _ => accept()?,
        };
        self.profile = intent.profile.clone();
        let failure = pending.or_else(|| self.store.finish_application_intent(active, &intent).err().map(|e| e.to_string()));
        self.state = failure.as_ref().map_or(PlayerProfilePersistenceState::Clean, |issue| PlayerProfilePersistenceState::ApplicationRecoveryPending { issue: issue.clone() });
        self.recovery_notification_pending = false;
        Ok(AcceptedApplication { revision: intent.active.revision, recovery_pending: failure.is_some() })
    }

    pub fn recover_pending_application(
        &mut self,
        active: &ActiveRunStore,
    ) -> Result<bool, PlayerProfileStoreError> {
        self.store.protect_originals()?;
        active
            .protect_versions()
            .map_err(PlayerProfileStoreError::ActiveRun)?;
        let Some(intent) = self.store.load_application_intent()? else {
            // The only accepted path that clears the journal does so after both
            // latest recovery copies are synced. A failed final directory sync
            // may leave this in-memory state pending with no journal to replay.
            if matches!(
                self.state,
                PlayerProfilePersistenceState::ApplicationRecoveryPending { .. }
            ) {
                self.store.sync_root()?;
                self.state = PlayerProfilePersistenceState::Clean;
            }
            return Ok(false);
        };
        self.state = PlayerProfilePersistenceState::ApplicationRecoveryPending {
            issue: "Accepted application is waiting for durable recovery".into(),
        };
        if let Err(error) = self.store.finish_application_intent(active, &intent) {
            self.state = PlayerProfilePersistenceState::ApplicationRecoveryPending {
                issue: error.to_string(),
            };
            return Err(error);
        }
        self.profile.adopt_accepted(intent.profile).map_err(PlayerProfileStoreError::InvalidEnvelope)?;
        self.state = PlayerProfilePersistenceState::Clean;
        self.recovery_notification_pending = false;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(value: u32, loads: u8) -> StoredCheckpoint {
        StoredCheckpoint {
            core_payload_json: serde_json::to_vec(
                &serde_json::json!({"version":12,"fixture_value":value}),
            )
            .unwrap(),
            loads_used: loads,
            run_peak_certified_power: value as u64 * 100,
        }
    }
    fn profile(value: u32) -> PlayerProfileState {
        PlayerProfileState {
            highest_floor_reached: value,
            ..Default::default()
        }
    }
    fn stores(root: &Path) -> (ActiveRunStore, PlayerProfilePersistence) {
        (
            ActiveRunStore::open(root).unwrap(),
            PlayerProfilePersistence::bootstrap(PlayerProfileStore::open(root).unwrap()),
        )
    }

    #[test]
    fn u04_conditional_intent_rejects_stale_choices_and_terminal_missing_aba() {
        let root = tempfile::tempdir().unwrap();
        let (active, mut persistence) = stores(root.path());
        let first = persistence
            .accept_application(
                &active,
                uuid::Uuid::nil(),
                Some(&checkpoint(1, 1)),
                &profile(1),
            )
            .unwrap();
        let chosen = persistence
            .accept_application(
                &active,
                first.revision,
                Some(&checkpoint(2, 2)),
                &profile(2),
            )
            .unwrap();
        let current = fs::read(root.path().join("active_run.json")).unwrap();
        let error = persistence
            .accept_application(
                &active,
                first.revision,
                Some(&checkpoint(3, 1)),
                &profile(3),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            PlayerProfileStoreError::ActiveRun(
                crate::persistence::active_run_store::ActiveRunStoreError::RevisionConflict { .. }
            )
        ));
        assert_eq!(
            fs::read(root.path().join("active_run.json")).unwrap(),
            current
        );
        assert_eq!(persistence.profile().unwrap(), profile(2));
        let terminal = persistence
            .accept_application(&active, chosen.revision, None, &profile(2))
            .unwrap();
        assert_ne!(terminal.revision, uuid::Uuid::nil());
        assert_eq!(
            active.load().unwrap(),
            crate::persistence::active_run_store::ActiveRunLoad::Missing
        );
        assert!(persistence
            .accept_application(
                &active,
                uuid::Uuid::nil(),
                Some(&checkpoint(4, 0)),
                &profile(4)
            )
            .is_err());
        assert!(persistence
            .accept_application(
                &active,
                chosen.revision,
                Some(&checkpoint(4, 0)),
                &profile(4)
            )
            .is_err());
        persistence
            .accept_application(
                &active,
                terminal.revision,
                Some(&checkpoint(4, 0)),
                &profile(4),
            )
            .unwrap();
        assert_eq!(active.snapshot().unwrap().checkpoint.unwrap().loads_used, 0);
    }

    #[test]
    fn u04_intent_io_failure_before_and_after_acceptance_has_distinct_recovery() {
        let root = tempfile::tempdir().unwrap();
        let (active, mut persistence) = stores(root.path());
        let before = persistence
            .accept_application(
                &active,
                uuid::Uuid::nil(),
                Some(&checkpoint(1, 1)),
                &profile(1),
            )
            .unwrap();
        fs::create_dir(root.path().join(JOURNAL_TEMP_FILE)).unwrap();
        assert!(persistence
            .accept_application(
                &active,
                before.revision,
                Some(&checkpoint(2, 2)),
                &profile(2)
            )
            .is_err());
        assert_eq!(active.snapshot().unwrap().revision, before.revision);
        assert_eq!(persistence.profile().unwrap(), profile(1));
        assert!(!root.path().join(JOURNAL_FILE).exists());
        fs::remove_dir(root.path().join(JOURNAL_TEMP_FILE)).unwrap();
        fs::create_dir(root.path().join(TEMP_FILE)).unwrap();
        let accepted = persistence
            .accept_application(
                &active,
                before.revision,
                Some(&checkpoint(2, 2)),
                &profile(2),
            )
            .unwrap();
        assert!(accepted.recovery_pending);
        assert!(matches!(
            persistence.state(),
            PlayerProfilePersistenceState::ApplicationRecoveryPending { .. }
        ));
        assert_eq!(active.snapshot().unwrap().revision, before.revision);
        assert!(root.path().join(JOURNAL_FILE).exists());
        assert!(persistence
            .accept_application(
                &active,
                before.revision,
                Some(&checkpoint(3, 3)),
                &profile(3)
            )
            .is_err());
        fs::remove_dir(root.path().join(TEMP_FILE)).unwrap();
        assert!(persistence.recover_pending_application(&active).unwrap());
        assert_eq!(active.snapshot().unwrap().revision, accepted.revision);
        assert_eq!(
            active.snapshot().unwrap().checkpoint,
            Some(checkpoint(2, 2))
        );
        assert_eq!(persistence.profile().unwrap(), profile(2));
        assert!(!persistence.recover_pending_application(&active).unwrap());
    }

    #[test]
    fn u04_real_process_crash_at_every_intent_and_aggregate_publication_stage() {
        const CHILD_ROOT: &str = "U04_INTENT_CHILD_ROOT";
        if let Some(root) = std::env::var_os(CHILD_ROOT) {
            let (active, mut persistence) = stores(Path::new(&root));
            let before = active.snapshot().unwrap().revision;
            persistence
                .accept_application(&active, before, Some(&checkpoint(7, 3)), &profile(7))
                .unwrap();
            panic!("requested crash stage was not visited");
        }
        let stages = [
            "intent_temp_synced",
            "intent_renamed",
            "intent_synced",
            "profile_temp_synced",
            "profile_renamed",
            "profile_synced",
            "profile_backup_temp_synced",
            "profile_backup_synced",
            "active_temp_synced",
            "active_renamed",
            "active_synced",
            "active_backup_temp_synced",
            "active_backup_synced",
            "before_intent_clear",
            "intent_removed",
            "intent_clear_synced",
        ];
        for stage in stages {
            let root = tempfile::tempdir().unwrap();
            {
                let (active, mut persistence) = stores(root.path());
                persistence
                    .accept_application(
                        &active,
                        uuid::Uuid::nil(),
                        Some(&checkpoint(1, 1)),
                        &profile(1),
                    )
                    .unwrap();
            }
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "persistence::player_profile_store::application_intent::tests::u04_real_process_crash_at_every_intent_and_aggregate_publication_stage", "--nocapture"])
                .env(CHILD_ROOT, root.path()).env("U04_DURABLE_CRASH_AT", stage).output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(86),
                "{stage}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let (active, mut persistence) = stores(root.path());
            persistence.recover_pending_application(&active).unwrap();
            let (value, loads) = if stage == "intent_temp_synced" {
                (1, 1)
            } else {
                (7, 3)
            };
            assert_eq!(
                active.snapshot().unwrap().checkpoint,
                Some(checkpoint(value, loads)),
                "{stage}"
            );
            assert_eq!(persistence.profile().unwrap(), profile(value), "{stage}");
            assert!(
                !persistence.recover_pending_application(&active).unwrap(),
                "{stage}"
            );
            for (current, backup) in [
                ("active_run.json", "active_run.backup.json"),
                ("player_profile.json", "player_profile.backup.json"),
            ] {
                assert_eq!(
                    fs::read(root.path().join(current)).unwrap(),
                    fs::read(root.path().join(backup)).unwrap(),
                    "{stage}/{current}"
                );
            }
            // Damaging the primary after completed recovery must retain exactly
            // the latest confirmed choice and all three Load uses.
            fs::write(root.path().join("active_run.json"), b"{").unwrap();
            assert_eq!(
                active.snapshot().unwrap().checkpoint,
                Some(checkpoint(value, loads)),
                "backup after {stage}"
            );
        }
    }

    #[test]
    fn u04_incompatible_current_backup_legacy_and_journal_originals_are_never_overwritten() {
        for (file, bytes) in [
            ("active_run.json", br#"{"version":4}"#.as_slice()),
            ("active_run.backup.json", br#"{"version":99}"#.as_slice()),
            ("player_profile.json", br#"{"version":99}"#.as_slice()),
            ("player_profile.backup.json", br#"{"version":2}"#.as_slice()),
            ("encounter_compendium.json", b"legacy".as_slice()),
            (
                "standard_completion.pending.json",
                br#"{"version":3}"#.as_slice(),
            ),
            (JOURNAL_FILE, br#"{"intent":{"version":99}}"#.as_slice()),
            (JOURNAL_FILE, b"{".as_slice()),
        ] {
            let root = tempfile::tempdir().unwrap();
            let (active, mut persistence) = stores(root.path());
            let before = persistence
                .accept_application(
                    &active,
                    uuid::Uuid::nil(),
                    Some(&checkpoint(1, 1)),
                    &profile(1),
                )
                .unwrap();
            fs::write(root.path().join(file), bytes).unwrap();
            let files = [
                "active_run.json",
                "active_run.backup.json",
                "player_profile.json",
                "player_profile.backup.json",
                file,
            ]
            .into_iter()
            .map(|file| (file, fs::read(root.path().join(file)).unwrap()))
            .collect::<Vec<_>>();
            assert!(
                persistence
                    .accept_application(
                        &active,
                        before.revision,
                        Some(&checkpoint(2, 2)),
                        &profile(2)
                    )
                    .is_err(),
                "{file}"
            );
            assert!(
                persistence.recover_pending_application(&active).is_err(),
                "{file}"
            );
            if file.starts_with("active_run") {
                assert!(active.reset_corrupt().is_err());
            } else {
                assert!(persistence.reset().is_err());
            }
            for (file, original) in files {
                assert_eq!(
                    fs::read(root.path().join(file)).unwrap(),
                    original,
                    "{file}"
                );
            }
        }
    }
}
