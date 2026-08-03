use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use game_core::game::encounter_compendium::EncounterCompendiumProfileState;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{replace_file, sync_directory};

const PROFILE_VERSION: u32 = 1;
const CURRENT_FILE: &str = "encounter_compendium.json";
const BACKUP_FILE: &str = "encounter_compendium.backup.json";
const TEMP_FILE: &str = "encounter_compendium.tmp";
const BACKUP_TEMP_FILE: &str = "encounter_compendium.backup.tmp";
const LOCK_FILE: &str = "encounter_compendium.lock";

#[derive(Debug)]
pub enum EncounterCompendiumStoreError {
    Io(std::io::Error),
    InvalidEnvelope(String),
    UnsupportedVersion(u32),
    ChecksumMismatch,
    BothCopiesInvalid { current: String, backup: String },
    LockUnavailable(std::io::Error),
}

impl std::fmt::Display for EncounterCompendiumStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "encounter compendium filesystem error: {error}"),
            Self::InvalidEnvelope(error) => {
                write!(formatter, "invalid encounter compendium envelope: {error}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported encounter compendium version {version}")
            }
            Self::ChecksumMismatch => formatter.write_str("encounter compendium checksum mismatch"),
            Self::BothCopiesInvalid { current, backup } => write!(
                formatter,
                "both encounter compendium copies are invalid (current: {current}; backup: {backup})"
            ),
            Self::LockUnavailable(error) => write!(
                formatter,
                "encounter compendium single-writer lock unavailable: {error}"
            ),
        }
    }
}

impl std::error::Error for EncounterCompendiumStoreError {}

impl From<std::io::Error> for EncounterCompendiumStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncounterCompendiumLoad {
    Missing,
    Loaded {
        profile: EncounterCompendiumProfileState,
        recovered_from_backup: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnsignedEncounterCompendiumEnvelope {
    version: u32,
    profile: EncounterCompendiumProfileState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EncounterCompendiumEnvelope {
    version: u32,
    profile: EncounterCompendiumProfileState,
    checksum_sha256: String,
}

impl EncounterCompendiumEnvelope {
    fn encode(
        profile: &EncounterCompendiumProfileState,
    ) -> Result<Vec<u8>, EncounterCompendiumStoreError> {
        profile
            .validate_persisted()
            .map_err(EncounterCompendiumStoreError::InvalidEnvelope)?;
        let unsigned = UnsignedEncounterCompendiumEnvelope {
            version: PROFILE_VERSION,
            profile: profile.clone(),
        };
        let unsigned_bytes = serde_json::to_vec(&unsigned)
            .map_err(|error| EncounterCompendiumStoreError::InvalidEnvelope(error.to_string()))?;
        let envelope = Self {
            version: unsigned.version,
            profile: unsigned.profile,
            checksum_sha256: format!("{:x}", Sha256::digest(unsigned_bytes)),
        };
        serde_json::to_vec_pretty(&envelope)
            .map_err(|error| EncounterCompendiumStoreError::InvalidEnvelope(error.to_string()))
    }

    fn decode(
        bytes: &[u8],
    ) -> Result<EncounterCompendiumProfileState, EncounterCompendiumStoreError> {
        let envelope: Self = serde_json::from_slice(bytes)
            .map_err(|error| EncounterCompendiumStoreError::InvalidEnvelope(error.to_string()))?;
        if envelope.version != PROFILE_VERSION {
            return Err(EncounterCompendiumStoreError::UnsupportedVersion(
                envelope.version,
            ));
        }
        let unsigned = UnsignedEncounterCompendiumEnvelope {
            version: envelope.version,
            profile: envelope.profile,
        };
        let unsigned_bytes = serde_json::to_vec(&unsigned)
            .map_err(|error| EncounterCompendiumStoreError::InvalidEnvelope(error.to_string()))?;
        if envelope.checksum_sha256 != format!("{:x}", Sha256::digest(unsigned_bytes)) {
            return Err(EncounterCompendiumStoreError::ChecksumMismatch);
        }
        unsigned
            .profile
            .validate_persisted()
            .map_err(EncounterCompendiumStoreError::InvalidEnvelope)?;
        Ok(unsigned.profile)
    }
}

pub struct EncounterCompendiumStore {
    root: PathBuf,
    _lock: File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncounterCompendiumPersistenceState {
    Clean,
    RecoveredFromBackup,
    Dirty { issue: String },
    Fatal { issue: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterCompendiumPersistOutcome {
    Unchanged,
    Saved,
    BecameDirty,
    StillDirty,
}

pub struct EncounterCompendiumPersistence {
    store: EncounterCompendiumStore,
    profile: EncounterCompendiumProfileState,
    state: EncounterCompendiumPersistenceState,
    recovery_notification_pending: bool,
}

impl EncounterCompendiumPersistence {
    pub fn bootstrap(store: EncounterCompendiumStore) -> Self {
        match store.load() {
            Ok(EncounterCompendiumLoad::Missing) => Self {
                store,
                profile: EncounterCompendiumProfileState::default(),
                state: EncounterCompendiumPersistenceState::Clean,
                recovery_notification_pending: false,
            },
            Ok(EncounterCompendiumLoad::Loaded {
                profile,
                recovered_from_backup,
            }) => Self {
                store,
                profile,
                state: if recovered_from_backup {
                    EncounterCompendiumPersistenceState::RecoveredFromBackup
                } else {
                    EncounterCompendiumPersistenceState::Clean
                },
                recovery_notification_pending: recovered_from_backup,
            },
            Err(error) => Self {
                store,
                profile: EncounterCompendiumProfileState::default(),
                state: EncounterCompendiumPersistenceState::Fatal {
                    issue: error.to_string(),
                },
                recovery_notification_pending: false,
            },
        }
    }

    pub fn profile(&self) -> Result<EncounterCompendiumProfileState, &str> {
        match &self.state {
            EncounterCompendiumPersistenceState::Fatal { issue } => Err(issue),
            _ => Ok(self.profile.clone()),
        }
    }

    pub fn state(&self) -> &EncounterCompendiumPersistenceState {
        &self.state
    }

    pub fn take_recovery_notification(&mut self) -> bool {
        std::mem::take(&mut self.recovery_notification_pending)
    }

    pub fn merge_and_persist(
        &mut self,
        profile: EncounterCompendiumProfileState,
    ) -> EncounterCompendiumPersistOutcome {
        if matches!(
            self.state,
            EncounterCompendiumPersistenceState::Fatal { .. }
        ) {
            return EncounterCompendiumPersistOutcome::Unchanged;
        }
        let before = self.profile.clone();
        self.profile.merge(profile);
        let changed = self.profile != before;
        let retry_needed = matches!(
            self.state,
            EncounterCompendiumPersistenceState::Dirty { .. }
                | EncounterCompendiumPersistenceState::RecoveredFromBackup
        );
        if !changed && !retry_needed {
            return EncounterCompendiumPersistOutcome::Unchanged;
        }
        let was_dirty = matches!(
            self.state,
            EncounterCompendiumPersistenceState::Dirty { .. }
        );
        match self.store.publish(&self.profile) {
            Ok(()) => {
                self.state = EncounterCompendiumPersistenceState::Clean;
                self.recovery_notification_pending = false;
                EncounterCompendiumPersistOutcome::Saved
            }
            Err(error) => {
                self.state = EncounterCompendiumPersistenceState::Dirty {
                    issue: error.to_string(),
                };
                if was_dirty {
                    EncounterCompendiumPersistOutcome::StillDirty
                } else {
                    EncounterCompendiumPersistOutcome::BecameDirty
                }
            }
        }
    }

    pub fn retry(&mut self) -> EncounterCompendiumPersistOutcome {
        self.merge_and_persist(EncounterCompendiumProfileState::default())
    }

    pub fn reload(&mut self) -> Result<(), EncounterCompendiumStoreError> {
        match self.store.load()? {
            EncounterCompendiumLoad::Missing => {
                self.profile = EncounterCompendiumProfileState::default();
                self.state = EncounterCompendiumPersistenceState::Clean;
                self.recovery_notification_pending = false;
            }
            EncounterCompendiumLoad::Loaded {
                profile,
                recovered_from_backup,
            } => {
                self.profile = profile;
                self.state = if recovered_from_backup {
                    EncounterCompendiumPersistenceState::RecoveredFromBackup
                } else {
                    EncounterCompendiumPersistenceState::Clean
                };
                self.recovery_notification_pending = recovered_from_backup;
            }
        }
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), EncounterCompendiumStoreError> {
        self.store.reset()?;
        self.profile = EncounterCompendiumProfileState::default();
        self.state = EncounterCompendiumPersistenceState::Clean;
        self.recovery_notification_pending = false;
        Ok(())
    }
}

impl EncounterCompendiumStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, EncounterCompendiumStoreError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(root.join(LOCK_FILE))?;
        lock.try_lock_exclusive()
            .map_err(EncounterCompendiumStoreError::LockUnavailable)?;
        Ok(Self { root, _lock: lock })
    }

    pub fn load(&self) -> Result<EncounterCompendiumLoad, EncounterCompendiumStoreError> {
        match self.read_decoded(CURRENT_FILE) {
            Ok(Some(profile)) => Ok(EncounterCompendiumLoad::Loaded {
                profile,
                recovered_from_backup: false,
            }),
            Ok(None) => match self.read_decoded(BACKUP_FILE)? {
                Some(profile) => Ok(EncounterCompendiumLoad::Loaded {
                    profile,
                    recovered_from_backup: true,
                }),
                None => Ok(EncounterCompendiumLoad::Missing),
            },
            Err(error @ EncounterCompendiumStoreError::UnsupportedVersion(_)) => Err(error),
            Err(current_error) => match self.read_decoded(BACKUP_FILE) {
                Ok(Some(profile)) => Ok(EncounterCompendiumLoad::Loaded {
                    profile,
                    recovered_from_backup: true,
                }),
                Ok(None) => Err(EncounterCompendiumStoreError::BothCopiesInvalid {
                    current: current_error.to_string(),
                    backup: "missing".to_string(),
                }),
                Err(error @ EncounterCompendiumStoreError::UnsupportedVersion(_)) => Err(error),
                Err(backup_error) => Err(EncounterCompendiumStoreError::BothCopiesInvalid {
                    current: current_error.to_string(),
                    backup: backup_error.to_string(),
                }),
            },
        }
    }

    pub fn publish(
        &self,
        profile: &EncounterCompendiumProfileState,
    ) -> Result<(), EncounterCompendiumStoreError> {
        let bytes = EncounterCompendiumEnvelope::encode(profile)?;
        EncounterCompendiumEnvelope::decode(&bytes)?;
        self.write_synced(TEMP_FILE, &bytes)?;

        if let Ok(Some(previous_profile)) = self.read_decoded(CURRENT_FILE) {
            let backup_bytes = EncounterCompendiumEnvelope::encode(&previous_profile)?;
            EncounterCompendiumEnvelope::decode(&backup_bytes)?;
            self.write_synced(BACKUP_TEMP_FILE, &backup_bytes)?;
            replace_file(&self.path(BACKUP_TEMP_FILE), &self.path(BACKUP_FILE))?;
        }

        replace_file(&self.path(TEMP_FILE), &self.path(CURRENT_FILE))?;
        self.sync_root()
    }

    pub fn reset(&self) -> Result<(), EncounterCompendiumStoreError> {
        self.publish(&EncounterCompendiumProfileState::default())?;
        for file in [BACKUP_FILE, BACKUP_TEMP_FILE] {
            let path = self.path(file);
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        self.sync_root()
    }

    fn read_decoded(
        &self,
        file_name: &str,
    ) -> Result<Option<EncounterCompendiumProfileState>, EncounterCompendiumStoreError> {
        let path = self.path(file_name);
        if !path.exists() {
            return Ok(None);
        }
        let mut bytes = Vec::new();
        File::open(path)?.read_to_end(&mut bytes)?;
        EncounterCompendiumEnvelope::decode(&bytes).map(Some)
    }

    fn write_synced(
        &self,
        file_name: &str,
        bytes: &[u8],
    ) -> Result<(), EncounterCompendiumStoreError> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(self.path(file_name))?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }

    fn sync_root(&self) -> Result<(), EncounterCompendiumStoreError> {
        sync_directory(&self.root).map_err(EncounterCompendiumStoreError::from)
    }

    fn path(&self, file_name: &str) -> PathBuf {
        self.root.join(file_name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn profile(id: &str, defeated: bool) -> EncounterCompendiumProfileState {
        EncounterCompendiumProfileState {
            discovered_abnormality_ids: BTreeSet::from([id.to_string()]),
            defeated_abnormality_ids: defeated
                .then(|| BTreeSet::from([id.to_string()]))
                .unwrap_or_default(),
            discovered_corrosion_profile_ids: BTreeSet::new(),
            defeated_corrosion_profile_ids: BTreeSet::new(),
        }
    }

    #[test]
    fn missing_publish_load_backup_recovery_and_reset_use_real_files() {
        let directory = tempfile::tempdir().unwrap();
        let store = EncounterCompendiumStore::open(directory.path()).unwrap();
        assert_eq!(store.load().unwrap(), EncounterCompendiumLoad::Missing);

        let first = profile("dormant_first", false);
        let second = profile("dormant_second", true);
        store.publish(&first).unwrap();
        store.publish(&second).unwrap();
        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
        assert_eq!(
            store.load().unwrap(),
            EncounterCompendiumLoad::Loaded {
                profile: first,
                recovered_from_backup: true,
            }
        );

        store.reset().unwrap();
        assert_eq!(
            store.load().unwrap(),
            EncounterCompendiumLoad::Loaded {
                profile: EncounterCompendiumProfileState::default(),
                recovered_from_backup: false,
            }
        );
        assert!(!directory.path().join(BACKUP_FILE).exists());
    }

    #[test]
    fn checksum_unsupported_dual_corrupt_and_single_writer_are_explicit() {
        let directory = tempfile::tempdir().unwrap();
        let store = EncounterCompendiumStore::open(directory.path()).unwrap();
        store.publish(&profile("known", false)).unwrap();
        let current = directory.path().join(CURRENT_FILE);
        let mut envelope: serde_json::Value =
            serde_json::from_slice(&fs::read(&current).unwrap()).unwrap();
        envelope["profile"]["discovered_abnormality_ids"] = serde_json::json!(["tampered"]);
        fs::write(&current, serde_json::to_vec(&envelope).unwrap()).unwrap();
        assert!(matches!(
            store.load(),
            Err(EncounterCompendiumStoreError::BothCopiesInvalid { .. })
        ));

        envelope["version"] = serde_json::json!(99);
        fs::write(&current, serde_json::to_vec(&envelope).unwrap()).unwrap();
        assert!(matches!(
            store.load(),
            Err(EncounterCompendiumStoreError::UnsupportedVersion(99))
        ));
        assert!(matches!(
            EncounterCompendiumStore::open(directory.path()),
            Err(EncounterCompendiumStoreError::LockUnavailable(_))
        ));
    }

    #[test]
    fn publish_failure_keeps_previous_current_and_does_not_rotate_corruption() {
        let directory = tempfile::tempdir().unwrap();
        let store = EncounterCompendiumStore::open(directory.path()).unwrap();
        let previous = profile("previous", false);
        store.publish(&previous).unwrap();
        fs::create_dir(directory.path().join(TEMP_FILE)).unwrap();
        assert!(store.publish(&profile("next", true)).is_err());
        assert_eq!(
            store.load().unwrap(),
            EncounterCompendiumLoad::Loaded {
                profile: previous,
                recovered_from_backup: false,
            }
        );
    }

    #[test]
    fn runtime_keeps_latest_union_dirty_and_retries_full_profile() {
        let directory = tempfile::tempdir().unwrap();
        let store = EncounterCompendiumStore::open(directory.path()).unwrap();
        let mut persistence = EncounterCompendiumPersistence::bootstrap(store);
        fs::create_dir(directory.path().join(TEMP_FILE)).unwrap();
        assert_eq!(
            persistence.merge_and_persist(profile("first", false)),
            EncounterCompendiumPersistOutcome::BecameDirty
        );
        assert!(matches!(
            persistence.state(),
            EncounterCompendiumPersistenceState::Dirty { .. }
        ));
        persistence.profile.merge(profile("second", true));
        fs::remove_dir(directory.path().join(TEMP_FILE)).unwrap();
        assert_eq!(
            persistence.retry(),
            EncounterCompendiumPersistOutcome::Saved
        );
        let loaded = persistence.store.load().unwrap();
        let EncounterCompendiumLoad::Loaded { profile, .. } = loaded else {
            panic!("retry must publish the latest full profile");
        };
        assert_eq!(
            profile.discovered_abnormality_ids,
            BTreeSet::from(["first".to_string(), "second".to_string()])
        );
        assert_eq!(
            profile.defeated_abnormality_ids,
            BTreeSet::from(["second".to_string()])
        );
    }
}
