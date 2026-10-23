use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use game_core::game::encounter_compendium::PlayerProfileState;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{replace_file, sync_directory};
use super::mod_requirements::ApprovedStoreGate;
use std::sync::Arc;

mod application_intent;
pub use application_intent::AcceptedApplication;

const PROFILE_VERSION: u32 = 6;
const CURRENT_FILE: &str = "player_profile.json";
const BACKUP_FILE: &str = "player_profile.backup.json";
const TEMP_FILE: &str = "player_profile.tmp";
const BACKUP_TEMP_FILE: &str = "player_profile.backup.tmp";
const LOCK_FILE: &str = "player_profile.lock";
const LEGACY_CURRENT_FILE: &str = "encounter_compendium.json";
const LEGACY_BACKUP_FILE: &str = "encounter_compendium.backup.json";
const LEGACY_TEMP_FILE: &str = "encounter_compendium.tmp";
const LEGACY_BACKUP_TEMP_FILE: &str = "encounter_compendium.backup.tmp";
const STANDARD_COMPLETION_JOURNAL_FILE: &str = "standard_completion.pending.json";
const STANDARD_COMPLETION_JOURNAL_TEMP_FILE: &str = "standard_completion.pending.tmp";

#[derive(Debug)]
pub enum PlayerProfileStoreError {
    Io(std::io::Error),
    InvalidEnvelope(String),
    UnsupportedVersion(u32),
    ChecksumMismatch,
    BothCopiesInvalid { current: String, backup: String },
    LockUnavailable(std::io::Error),
    ActiveRun(super::active_run_store::ActiveRunStoreError),
    ProtectedOriginal(String),
}

impl std::fmt::Display for PlayerProfileStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ActiveRun(error) => write!(formatter, "{error}"),
            Self::ProtectedOriginal(file) => {
                write!(formatter, "unsupported original file is protected: {file}")
            }
            Self::Io(error) => write!(formatter, "player profile filesystem error: {error}"),
            Self::InvalidEnvelope(error) => {
                write!(formatter, "invalid player profile envelope: {error}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported player profile version {version}")
            }
            Self::ChecksumMismatch => formatter.write_str("player profile checksum mismatch"),
            Self::BothCopiesInvalid { current, backup } => write!(
                formatter,
                "both player profile copies are invalid (current: {current}; backup: {backup})"
            ),
            Self::LockUnavailable(error) => write!(
                formatter,
                "player profile single-writer lock unavailable: {error}"
            ),
        }
    }
}

impl std::error::Error for PlayerProfileStoreError {}

impl From<std::io::Error> for PlayerProfileStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerProfileLoad {
    Missing,
    Loaded {
        profile: PlayerProfileState,
        recovered_from_backup: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnsignedPlayerProfileEnvelope {
    version: u32,
    profile: PlayerProfileState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlayerProfileEnvelope {
    version: u32,
    profile: PlayerProfileState,
    checksum_sha256: String,
}

impl PlayerProfileEnvelope {
    fn encode(profile: &PlayerProfileState) -> Result<Vec<u8>, PlayerProfileStoreError> {
        profile
            .validate_persisted()
            .map_err(PlayerProfileStoreError::InvalidEnvelope)?;
        let unsigned = UnsignedPlayerProfileEnvelope {
            version: PROFILE_VERSION,
            profile: profile.clone(),
        };
        let unsigned_bytes = serde_json::to_vec(&unsigned)
            .map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        let envelope = Self {
            version: unsigned.version,
            profile: unsigned.profile,
            checksum_sha256: format!("{:x}", Sha256::digest(unsigned_bytes)),
        };
        let bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        if bytes.len() > super::mod_requirements::PROFILE_MAX_BYTES { return Err(PlayerProfileStoreError::InvalidEnvelope("profile exceeds supported byte ceiling".into())); }
        Ok(bytes)
    }

    fn decode(bytes: &[u8]) -> Result<PlayerProfileState, PlayerProfileStoreError> {
        if bytes.len() > super::mod_requirements::PROFILE_MAX_BYTES { return Err(PlayerProfileStoreError::InvalidEnvelope("profile exceeds supported byte ceiling".into())); }
        #[derive(Deserialize)]
        struct Header {
            version: u32,
        }
        let header: Header = serde_json::from_slice(bytes)
            .map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        if header.version != PROFILE_VERSION {
            return Err(PlayerProfileStoreError::UnsupportedVersion(header.version));
        }
        let envelope: Self = serde_json::from_slice(bytes)
            .map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        if envelope.version != PROFILE_VERSION {
            return Err(PlayerProfileStoreError::UnsupportedVersion(
                envelope.version,
            ));
        }
        let unsigned = UnsignedPlayerProfileEnvelope {
            version: envelope.version,
            profile: envelope.profile,
        };
        let unsigned_bytes = serde_json::to_vec(&unsigned)
            .map_err(|error| PlayerProfileStoreError::InvalidEnvelope(error.to_string()))?;
        if envelope.checksum_sha256 != format!("{:x}", Sha256::digest(unsigned_bytes)) {
            return Err(PlayerProfileStoreError::ChecksumMismatch);
        }
        unsigned
            .profile
            .validate_persisted()
            .map_err(PlayerProfileStoreError::InvalidEnvelope)?;
        Ok(unsigned.profile)
    }
}

pub struct PlayerProfileStore {
    root: PathBuf,
    _lock: File,
    gate: Option<Arc<ApprovedStoreGate>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerProfilePersistenceState {
    Clean,
    RecoveredFromBackup,
    Dirty { issue: String },
    ApplicationRecoveryPending { issue: String },
    Fatal { issue: String },
    Protected { issue: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerProfilePersistOutcome {
    Unchanged,
    Saved,
    BecameDirty,
    StillDirty,
}

pub struct PlayerProfilePersistence {
    store: PlayerProfileStore,
    profile: PlayerProfileState,
    state: PlayerProfilePersistenceState,
    recovery_notification_pending: bool,
}

impl PlayerProfilePersistence {
    pub fn bootstrap(store: PlayerProfileStore) -> Self {
        match store.load() {
            Ok(PlayerProfileLoad::Missing) => Self {
                store,
                profile: PlayerProfileState::default(),
                state: PlayerProfilePersistenceState::Clean,
                recovery_notification_pending: false,
            },
            Ok(PlayerProfileLoad::Loaded {
                profile,
                recovered_from_backup,
            }) => Self {
                store,
                profile,
                state: if recovered_from_backup {
                    PlayerProfilePersistenceState::RecoveredFromBackup
                } else {
                    PlayerProfilePersistenceState::Clean
                },
                recovery_notification_pending: recovered_from_backup,
            },
            Err(error) => {
                let protected = store.protect_originals().is_err()
                    || store.require_no_application_intent().is_err();
                let state = if protected {
                    PlayerProfilePersistenceState::Protected {
                        issue: error.to_string(),
                    }
                } else {
                    PlayerProfilePersistenceState::Fatal {
                        issue: error.to_string(),
                    }
                };
                Self {
                    store,
                    profile: PlayerProfileState::default(),
                    state,
                    recovery_notification_pending: false,
                }
            }
        }
    }

    pub fn profile(&self) -> Result<PlayerProfileState, &str> {
        match &self.state {
            PlayerProfilePersistenceState::ApplicationRecoveryPending { issue }
            | PlayerProfilePersistenceState::Protected { issue }
            | PlayerProfilePersistenceState::Fatal { issue } => Err(issue),
            _ => Ok(self.profile.clone()),
        }
    }

    pub fn state(&self) -> &PlayerProfilePersistenceState {
        &self.state
    }

    pub fn take_recovery_notification(&mut self) -> bool {
        std::mem::take(&mut self.recovery_notification_pending)
    }

    pub fn merge_and_persist<P: Into<PlayerProfileState>>(
        &mut self,
        profile: P,
    ) -> Result<PlayerProfilePersistOutcome, PlayerProfileStoreError> {
        if matches!(
            self.state,
            PlayerProfilePersistenceState::ApplicationRecoveryPending { .. }
                | PlayerProfilePersistenceState::Protected { .. }
                | PlayerProfilePersistenceState::Fatal { .. }
        ) {
            return Ok(PlayerProfilePersistOutcome::Unchanged);
        }
        let before = self.profile.clone();
        self.profile.merge(profile.into()).map_err(PlayerProfileStoreError::InvalidEnvelope)?;
        let changed = self.profile != before;
        let retry_needed = matches!(
            self.state,
            PlayerProfilePersistenceState::Dirty { .. }
                | PlayerProfilePersistenceState::RecoveredFromBackup
        );
        if !changed && !retry_needed {
            return Ok(PlayerProfilePersistOutcome::Unchanged);
        }
        let was_dirty = matches!(self.state, PlayerProfilePersistenceState::Dirty { .. });
        Ok(match self.store.publish(&self.profile) {
            Ok(()) => {
                self.state = PlayerProfilePersistenceState::Clean;
                self.recovery_notification_pending = false;
                PlayerProfilePersistOutcome::Saved
            }
            Err(error) => {
                self.state = PlayerProfilePersistenceState::Dirty {
                    issue: error.to_string(),
                };
                if was_dirty {
                    PlayerProfilePersistOutcome::StillDirty
                } else {
                    PlayerProfilePersistOutcome::BecameDirty
                }
            }
        })
    }

    pub fn retry(&mut self) -> Result<PlayerProfilePersistOutcome, PlayerProfileStoreError> {
        self.merge_and_persist(self.profile.clone())
    }

    pub fn reload(&mut self) -> Result<(), PlayerProfileStoreError> {
        match self.store.load()? {
            PlayerProfileLoad::Missing => {
                self.profile = PlayerProfileState::default();
                self.state = PlayerProfilePersistenceState::Clean;
                self.recovery_notification_pending = false;
            }
            PlayerProfileLoad::Loaded {
                profile,
                recovered_from_backup,
            } => {
                self.profile = profile;
                self.state = if recovered_from_backup {
                    PlayerProfilePersistenceState::RecoveredFromBackup
                } else {
                    PlayerProfilePersistenceState::Clean
                };
                self.recovery_notification_pending = recovered_from_backup;
            }
        }
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), PlayerProfileStoreError> {
        if self.profile.lifecycle.active_expedition.is_some() { return Err(PlayerProfileStoreError::InvalidEnvelope("Continue and abandon the active expedition before resetting the profile".into())); }
        self.store.reset()?;
        self.profile = PlayerProfileState::default();
        self.state = PlayerProfilePersistenceState::Clean;
        self.recovery_notification_pending = false;
        Ok(())
    }
}

impl PlayerProfileStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, PlayerProfileStoreError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(root.join(LOCK_FILE))?;
        lock.try_lock_exclusive()
            .map_err(PlayerProfileStoreError::LockUnavailable)?;
        Ok(Self { root, _lock: lock, gate: None })
    }

    pub(crate) fn open_approved(root: impl AsRef<Path>, gate: Arc<ApprovedStoreGate>) -> Result<Self, PlayerProfileStoreError> {
        gate.preflight().map_err(PlayerProfileStoreError::ActiveRun)?;
        let mut store = Self::open(root)?;
        store.gate = Some(gate);
        store.protect_originals()?;
        Ok(store)
    }

    pub fn load(&self) -> Result<PlayerProfileLoad, PlayerProfileStoreError> {
        self.protect_originals()?;
        match self.read_decoded(CURRENT_FILE) {
            Ok(Some(profile)) => Ok(PlayerProfileLoad::Loaded {
                profile,
                recovered_from_backup: false,
            }),
            Ok(None) => match self.read_decoded(BACKUP_FILE)? {
                Some(profile) => Ok(PlayerProfileLoad::Loaded {
                    profile,
                    recovered_from_backup: true,
                }),
                None => Ok(PlayerProfileLoad::Missing),
            },
            Err(error @ PlayerProfileStoreError::UnsupportedVersion(_)) => Err(error),
            Err(current_error) => match self.read_decoded(BACKUP_FILE) {
                Ok(Some(profile)) => Ok(PlayerProfileLoad::Loaded {
                    profile,
                    recovered_from_backup: true,
                }),
                Ok(None) => Err(PlayerProfileStoreError::BothCopiesInvalid {
                    current: current_error.to_string(),
                    backup: "missing".to_string(),
                }),
                Err(error @ PlayerProfileStoreError::UnsupportedVersion(_)) => Err(error),
                Err(backup_error) => Err(PlayerProfileStoreError::BothCopiesInvalid {
                    current: current_error.to_string(),
                    backup: backup_error.to_string(),
                }),
            },
        }
    }

    pub fn publish<P: Clone + Into<PlayerProfileState>>(
        &self,
        profile: &P,
    ) -> Result<(), PlayerProfileStoreError> {
        self.protect_originals()?;
        self.require_no_application_intent()?;
        self.write_confirmed_profile(&profile.clone().into())
    }

    pub fn reset(&self) -> Result<(), PlayerProfileStoreError> {
        self.protect_originals()?;
        self.require_no_application_intent()?;
        self.write_confirmed_profile(&PlayerProfileState::default())
    }

    fn write_confirmed_profile(
        &self,
        profile: &PlayerProfileState,
    ) -> Result<(), PlayerProfileStoreError> {
        self.protect_originals()?;
        let bytes = PlayerProfileEnvelope::encode(profile)?;
        PlayerProfileEnvelope::decode(&bytes)?;
        self.write_synced(TEMP_FILE, &bytes)?;
        super::durable_crash_point("profile_temp_synced");
        replace_file(&self.path(TEMP_FILE), &self.path(CURRENT_FILE))?;
        super::durable_crash_point("profile_renamed");
        self.sync_root()?;
        super::durable_crash_point("profile_synced");
        self.write_synced(BACKUP_TEMP_FILE, &bytes)?;
        super::durable_crash_point("profile_backup_temp_synced");
        replace_file(&self.path(BACKUP_TEMP_FILE), &self.path(BACKUP_FILE))?;
        self.sync_root()?;
        super::durable_crash_point("profile_backup_synced");
        Ok(())
    }

    fn read_decoded(
        &self,
        file_name: &str,
    ) -> Result<Option<PlayerProfileState>, PlayerProfileStoreError> {
        let path = self.path(file_name);
        if !path.exists() {
            return Ok(None);
        }
        let mut bytes = Vec::new();
        File::open(path)?.read_to_end(&mut bytes)?;
        PlayerProfileEnvelope::decode(&bytes).map(Some)
    }

    fn write_synced(&self, file_name: &str, bytes: &[u8]) -> Result<(), PlayerProfileStoreError> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(self.path(file_name))?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }

    fn sync_root(&self) -> Result<(), PlayerProfileStoreError> {
        sync_directory(&self.root).map_err(PlayerProfileStoreError::from)
    }

    fn path(&self, file_name: &str) -> PathBuf {
        self.root.join(file_name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::persistence::active_run_store::StoredCheckpoint as ActiveRunCheckpointExport;
    use game_core::game::encounter_compendium::EncounterCompendiumProfileState;

    use super::*;
    use crate::persistence::active_run_store::{ActiveRunLoad, ActiveRunStore};

    fn profile(id: &str, _retired_defeated_flag: bool) -> EncounterCompendiumProfileState {
        EncounterCompendiumProfileState {
            registered_ids: BTreeSet::from([id.to_string()]),
        }
    }

    fn full_profile() -> EncounterCompendiumProfileState {
        EncounterCompendiumProfileState {
            registered_ids: BTreeSet::from([
                "dormant_abnormality".to_string(),
                "live_abnormality".to_string(),
                "dormant_corrosion".to_string(),
                "live_corrosion".to_string(),
            ]),
        }
    }

    fn active_checkpoint() -> ActiveRunCheckpointExport {
        ActiveRunCheckpointExport {
            core_payload_json: br#"{"version":12,"standard":"in_progress"}"#.to_vec(),
            loads_used: 0,
            run_peak_certified_power: 100,
        }
    }

    fn completed_standard_profile() -> PlayerProfileState {
        PlayerProfileState {
            facility_knowledge: Default::default(),
            encounter_compendium: full_profile(),
            highest_floor_reached: 3,
            longest_deathless_floor_streak: 3,
            all_live_final_bosses_defeated: false,
            lifecycle: Default::default(),
        }
    }

    #[test]
    fn missing_publish_load_backup_recovery_and_reset_use_real_files() {
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        assert_eq!(store.load().unwrap(), PlayerProfileLoad::Missing);

        let first = profile("dormant_first", false);
        let second = profile("dormant_second", true);
        store.publish(&first).unwrap();
        store.publish(&second).unwrap();
        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
        assert_eq!(
            store.load().unwrap(),
            PlayerProfileLoad::Loaded {
                profile: second.into(),
                recovered_from_backup: true,
            }
        );

        store.reset().unwrap();
        assert_eq!(
            store.load().unwrap(),
            PlayerProfileLoad::Loaded {
                profile: PlayerProfileState::default(),
                recovered_from_backup: false,
            }
        );
        assert_eq!(
            fs::read(directory.path().join(BACKUP_FILE)).unwrap(),
            fs::read(directory.path().join(CURRENT_FILE)).unwrap()
        );
    }

    #[test]
    fn checksum_unsupported_dual_corrupt_and_single_writer_are_explicit() {
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        store.publish(&profile("known", false)).unwrap();
        let current = directory.path().join(CURRENT_FILE);
        let mut envelope: serde_json::Value =
            serde_json::from_slice(&fs::read(&current).unwrap()).unwrap();
        envelope["profile"]["encounter_compendium"]["registered_ids"] =
            serde_json::json!(["tampered"]);
        fs::write(&current, serde_json::to_vec(&envelope).unwrap()).unwrap();
        fs::write(directory.path().join(BACKUP_FILE), b"corrupt backup").unwrap();
        assert!(matches!(
            store.load(),
            Err(PlayerProfileStoreError::BothCopiesInvalid { .. })
        ));

        envelope["version"] = serde_json::json!(99);
        fs::write(&current, serde_json::to_vec(&envelope).unwrap()).unwrap();
        assert!(matches!(
            store.load(),
            Err(PlayerProfileStoreError::UnsupportedVersion(99))
        ));
        assert!(matches!(
            PlayerProfileStore::open(directory.path()),
            Err(PlayerProfileStoreError::LockUnavailable(_))
        ));
    }

    #[test]
    fn publish_failure_keeps_previous_current_and_does_not_rotate_corruption() {
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        let previous = profile("previous", false);
        store.publish(&previous).unwrap();
        fs::create_dir(directory.path().join(TEMP_FILE)).unwrap();
        assert!(store.publish(&profile("next", true)).is_err());
        assert_eq!(
            store.load().unwrap(),
            PlayerProfileLoad::Loaded {
                profile: previous.into(),
                recovered_from_backup: false,
            }
        );
    }

    #[test]
    fn runtime_keeps_latest_registered_set_dirty_and_retries_full_profile() {
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        let mut persistence = PlayerProfilePersistence::bootstrap(store);
        fs::create_dir(directory.path().join(TEMP_FILE)).unwrap();
        assert_eq!(
            persistence.merge_and_persist(profile("first", false)).unwrap(),
            PlayerProfilePersistOutcome::BecameDirty
        );
        assert!(matches!(
            persistence.state(),
            PlayerProfilePersistenceState::Dirty { .. }
        ));
        persistence.profile.merge(profile("second", true)).unwrap();
        fs::remove_dir(directory.path().join(TEMP_FILE)).unwrap();
        assert_eq!(persistence.retry().unwrap(), PlayerProfilePersistOutcome::Saved);
        let loaded = persistence.store.load().unwrap();
        let PlayerProfileLoad::Loaded { profile, .. } = loaded else {
            panic!("retry must publish the latest full profile");
        };
        assert_eq!(
            profile.encounter_compendium.registered_ids,
            BTreeSet::from(["first".to_string(), "second".to_string()])
        );
    }

    #[test]
    fn retired_legacy_compendium_file_is_not_read_or_migrated() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join(LEGACY_CURRENT_FILE),
            br#"{"version":1,"profile":{"discovered_abnormality_ids":["retired"]}}"#,
        )
        .unwrap();

        let store = PlayerProfileStore::open(directory.path()).unwrap();
        let original = fs::read(directory.path().join(LEGACY_CURRENT_FILE)).unwrap();
        assert!(matches!(
            store.load(),
            Err(PlayerProfileStoreError::ProtectedOriginal(_))
        ));
        assert!(store.reset().is_err());
        assert!(store.publish(&PlayerProfileState::default()).is_err());
        let persistence = PlayerProfilePersistence::bootstrap(store);
        assert!(persistence.profile().is_err());
        assert!(matches!(
            persistence.state(),
            PlayerProfilePersistenceState::Protected { .. }
        ));
        assert_eq!(
            fs::read(directory.path().join(LEGACY_CURRENT_FILE)).unwrap(),
            original
        );
        assert!(directory.path().join(LEGACY_CURRENT_FILE).is_file());
        assert!(!directory.path().join(CURRENT_FILE).exists());
    }

    #[test]
    fn retired_version_two_profile_is_rejected_without_union_migration() {
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        let mut envelope: serde_json::Value = serde_json::from_slice(
            &PlayerProfileEnvelope::encode(&PlayerProfileState::default()).unwrap(),
        )
        .unwrap();
        envelope["version"] = serde_json::json!(2);
        fs::write(
            directory.path().join(CURRENT_FILE),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            store.load(),
            Err(PlayerProfileStoreError::UnsupportedVersion(2))
        ));
    }

    #[test]
    fn successor_profile_rejects_unknown_fields_and_ignores_interrupted_temp() {
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        let first: PlayerProfileState = profile("installed", false).into();
        let second: PlayerProfileState = profile("interrupted", true).into();
        store.publish(&first).unwrap();
        fs::write(
            directory.path().join(TEMP_FILE),
            PlayerProfileEnvelope::encode(&second).unwrap(),
        )
        .unwrap();
        assert_eq!(
            store.load().unwrap(),
            PlayerProfileLoad::Loaded {
                profile: first,
                recovered_from_backup: false,
            }
        );

        let current = directory.path().join(CURRENT_FILE);
        let mut envelope: serde_json::Value =
            serde_json::from_slice(&fs::read(&current).unwrap()).unwrap();
        envelope["profile"]["unknown_successor_field"] = serde_json::json!(true);
        fs::write(&current, serde_json::to_vec(&envelope).unwrap()).unwrap();
        fs::write(directory.path().join(BACKUP_FILE), b"corrupt backup").unwrap();
        assert!(matches!(
            store.load(),
            Err(PlayerProfileStoreError::BothCopiesInvalid { .. })
        ));
    }

    #[test]
    fn terminal_application_journal_closes_profile_and_active_run_after_restart() {
        let root = tempfile::tempdir().unwrap();
        let active = ActiveRunStore::open(root.path()).unwrap();
        active.publish(&active_checkpoint()).unwrap();
        let mut persistence =
            PlayerProfilePersistence::bootstrap(PlayerProfileStore::open(root.path()).unwrap());
        fs::create_dir(root.path().join(TEMP_FILE)).unwrap();
        let accepted = persistence
            .accept_application(
                &active,
                active.snapshot().unwrap().revision,
                None,
                &completed_standard_profile(),
            )
            .unwrap();
        assert!(accepted.recovery_pending);
        assert!(persistence.profile().is_err());
        assert!(matches!(
            active.load().unwrap(),
            ActiveRunLoad::Loaded { .. }
        ));
        drop(persistence);
        drop(active);
        fs::remove_dir(root.path().join(TEMP_FILE)).unwrap();
        let active = ActiveRunStore::open(root.path()).unwrap();
        let mut persistence =
            PlayerProfilePersistence::bootstrap(PlayerProfileStore::open(root.path()).unwrap());
        assert!(persistence.recover_pending_application(&active).unwrap());
        assert!(matches!(
            persistence.state(),
            PlayerProfilePersistenceState::Clean
        ));
        assert_eq!(active.load().unwrap(), ActiveRunLoad::Missing);
        assert_eq!(active.snapshot().unwrap().revision, accepted.revision);
        assert_eq!(persistence.profile().unwrap(), completed_standard_profile());
        assert!(!persistence.recover_pending_application(&active).unwrap());
    }

    #[test]
    fn terminal_application_preintent_failure_exposes_neither_side() {
        let root = tempfile::tempdir().unwrap();
        let active = ActiveRunStore::open(root.path()).unwrap();
        active.publish(&active_checkpoint()).unwrap();
        let before = active.snapshot().unwrap();
        let mut persistence =
            PlayerProfilePersistence::bootstrap(PlayerProfileStore::open(root.path()).unwrap());
        fs::create_dir(root.path().join("application_commit.pending.tmp")).unwrap();
        assert!(persistence
            .accept_application(
                &active,
                before.revision,
                None,
                &completed_standard_profile()
            )
            .is_err());
        assert_eq!(
            persistence.profile().unwrap(),
            PlayerProfileState::default()
        );
        assert_eq!(active.snapshot().unwrap(), before);
        assert!(!root.path().join("application_commit.pending.json").exists());
        assert!(!root.path().join(CURRENT_FILE).exists());
    }

    #[test]
    fn terminal_application_retries_failed_active_invalidation_from_journal() {
        let root = tempfile::tempdir().unwrap();
        let active = ActiveRunStore::open(root.path()).unwrap();
        active.publish(&active_checkpoint()).unwrap();
        let mut persistence =
            PlayerProfilePersistence::bootstrap(PlayerProfileStore::open(root.path()).unwrap());
        fs::create_dir(root.path().join("active_run.tmp")).unwrap();
        let accepted = persistence
            .accept_application(
                &active,
                active.snapshot().unwrap().revision,
                None,
                &completed_standard_profile(),
            )
            .unwrap();
        assert!(accepted.recovery_pending);
        assert!(persistence.recover_pending_application(&active).is_err());
        assert!(persistence.profile().is_err());
        assert!(root
            .path()
            .join("application_commit.pending.json")
            .is_file());
        fs::remove_dir(root.path().join("active_run.tmp")).unwrap();
        assert!(persistence.recover_pending_application(&active).unwrap());
        assert!(matches!(
            persistence.state(),
            PlayerProfilePersistenceState::Clean
        ));
        assert_eq!(active.load().unwrap(), ActiveRunLoad::Missing);
        assert_eq!(persistence.profile().unwrap(), completed_standard_profile());
    }

    #[test]
    fn explicit_reset_preserves_retired_profile_and_journal_originals() {
        for protected in [
            LEGACY_CURRENT_FILE,
            LEGACY_BACKUP_FILE,
            LEGACY_TEMP_FILE,
            LEGACY_BACKUP_TEMP_FILE,
            STANDARD_COMPLETION_JOURNAL_FILE,
            STANDARD_COMPLETION_JOURNAL_TEMP_FILE,
        ] {
            let root = tempfile::tempdir().unwrap();
            let store = PlayerProfileStore::open(root.path()).unwrap();
            store.publish(&completed_standard_profile()).unwrap();
            let current = fs::read(root.path().join(CURRENT_FILE)).unwrap();
            let backup = fs::read(root.path().join(BACKUP_FILE)).unwrap();
            fs::write(
                root.path().join(protected),
                b"protected historical original",
            )
            .unwrap();
            assert!(matches!(
                store.reset(),
                Err(PlayerProfileStoreError::ProtectedOriginal(_))
            ));
            assert_eq!(
                fs::read(root.path().join(protected)).unwrap(),
                b"protected historical original"
            );
            assert_eq!(fs::read(root.path().join(CURRENT_FILE)).unwrap(), current);
            assert_eq!(fs::read(root.path().join(BACKUP_FILE)).unwrap(), backup);
        }
    }
}
