use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use fs2::FileExt;
use game_core::game::world::ActiveRunCheckpointExport;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{replace_file, sync_directory};

const ACTIVE_RUN_VERSION: u32 = 3;
const CURRENT_FILE: &str = "active_run.json";
const BACKUP_FILE: &str = "active_run.backup.json";
const TEMP_FILE: &str = "active_run.tmp";
const BACKUP_TEMP_FILE: &str = "active_run.backup.tmp";
const INVALIDATED_FILE: &str = "active_run.invalidated";
const LOCK_FILE: &str = "active_run.lock";

#[derive(Debug)]
pub enum ActiveRunStoreError {
    Io(std::io::Error),
    InvalidEnvelope(String),
    UnsupportedVersion(u32),
    ChecksumMismatch,
    BothCopiesInvalid { current: String, backup: String },
    LockUnavailable(std::io::Error),
}

impl std::fmt::Display for ActiveRunStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "active-run filesystem error: {error}"),
            Self::InvalidEnvelope(error) => {
                write!(formatter, "invalid active-run envelope: {error}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported active-run version {version}")
            }
            Self::ChecksumMismatch => formatter.write_str("active-run checksum mismatch"),
            Self::BothCopiesInvalid { current, backup } => write!(
                formatter,
                "both active-run copies are invalid (current: {current}; backup: {backup})"
            ),
            Self::LockUnavailable(error) => {
                write!(
                    formatter,
                    "active-run single-writer lock unavailable: {error}"
                )
            }
        }
    }
}

impl std::error::Error for ActiveRunStoreError {}

impl From<std::io::Error> for ActiveRunStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveRunLoad {
    Missing,
    Loaded {
        checkpoint: ActiveRunCheckpointExport,
        recovered_from_backup: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActiveRunMonotonicMetadata {
    loads_used: u8,
    run_peak_certified_power: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnsignedActiveRunEnvelope {
    version: u32,
    core_payload_base64: String,
    monotonic: ActiveRunMonotonicMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActiveRunEnvelope {
    version: u32,
    core_payload_base64: String,
    monotonic: ActiveRunMonotonicMetadata,
    checksum_sha256: String,
}

impl ActiveRunEnvelope {
    fn encode(checkpoint: &ActiveRunCheckpointExport) -> Result<Vec<u8>, ActiveRunStoreError> {
        let unsigned = UnsignedActiveRunEnvelope {
            version: ACTIVE_RUN_VERSION,
            core_payload_base64: BASE64.encode(&checkpoint.core_payload_json),
            monotonic: ActiveRunMonotonicMetadata {
                loads_used: checkpoint.loads_used,
                run_peak_certified_power: checkpoint.run_peak_certified_power,
            },
        };
        let unsigned_bytes = serde_json::to_vec(&unsigned)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
        let envelope = Self {
            version: unsigned.version,
            core_payload_base64: unsigned.core_payload_base64,
            monotonic: unsigned.monotonic,
            checksum_sha256: format!("{:x}", Sha256::digest(unsigned_bytes)),
        };
        serde_json::to_vec_pretty(&envelope)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))
    }

    fn decode(bytes: &[u8]) -> Result<ActiveRunCheckpointExport, ActiveRunStoreError> {
        let envelope: Self = serde_json::from_slice(bytes)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
        if envelope.version != ACTIVE_RUN_VERSION {
            return Err(ActiveRunStoreError::UnsupportedVersion(envelope.version));
        }
        let unsigned = UnsignedActiveRunEnvelope {
            version: envelope.version,
            core_payload_base64: envelope.core_payload_base64,
            monotonic: envelope.monotonic,
        };
        let unsigned_bytes = serde_json::to_vec(&unsigned)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
        let expected = format!("{:x}", Sha256::digest(unsigned_bytes));
        if envelope.checksum_sha256 != expected {
            return Err(ActiveRunStoreError::ChecksumMismatch);
        }
        let core_payload_json = BASE64
            .decode(unsigned.core_payload_base64)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
        Ok(ActiveRunCheckpointExport {
            core_payload_json,
            loads_used: unsigned.monotonic.loads_used,
            run_peak_certified_power: unsigned.monotonic.run_peak_certified_power,
        })
    }
}

pub struct ActiveRunStore {
    root: PathBuf,
    _lock: File,
}

impl ActiveRunStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ActiveRunStoreError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(root.join(LOCK_FILE))?;
        lock.try_lock_exclusive()
            .map_err(ActiveRunStoreError::LockUnavailable)?;
        Ok(Self { root, _lock: lock })
    }

    pub fn load(&self) -> Result<ActiveRunLoad, ActiveRunStoreError> {
        if self.path(INVALIDATED_FILE).exists() {
            return Ok(ActiveRunLoad::Missing);
        }
        match self.read_decoded(CURRENT_FILE) {
            Ok(Some(checkpoint)) => Ok(ActiveRunLoad::Loaded {
                checkpoint,
                recovered_from_backup: false,
            }),
            Ok(None) => match self.read_decoded(BACKUP_FILE)? {
                Some(checkpoint) => {
                    self.restore_backup_to_current()?;
                    Ok(ActiveRunLoad::Loaded {
                        checkpoint,
                        recovered_from_backup: true,
                    })
                }
                None => Ok(ActiveRunLoad::Missing),
            },
            Err(current_error) => match self.read_decoded(BACKUP_FILE) {
                Ok(Some(checkpoint)) => {
                    self.restore_backup_to_current()?;
                    Ok(ActiveRunLoad::Loaded {
                        checkpoint,
                        recovered_from_backup: true,
                    })
                }
                Ok(None) => Err(ActiveRunStoreError::BothCopiesInvalid {
                    current: current_error.to_string(),
                    backup: "missing".to_string(),
                }),
                Err(backup_error) => Err(ActiveRunStoreError::BothCopiesInvalid {
                    current: current_error.to_string(),
                    backup: backup_error.to_string(),
                }),
            },
        }
    }

    pub fn publish(
        &self,
        checkpoint: &ActiveRunCheckpointExport,
    ) -> Result<(), ActiveRunStoreError> {
        let bytes = ActiveRunEnvelope::encode(checkpoint)?;
        ActiveRunEnvelope::decode(&bytes)?;
        self.write_synced(TEMP_FILE, &bytes)?;

        let invalidated = self.path(INVALIDATED_FILE);
        let replaces_invalidated_run = invalidated.exists();
        if !replaces_invalidated_run {
            if let Ok(Some(mut previous_checkpoint)) = self.read_decoded(CURRENT_FILE) {
                // The backup keeps the previous checkpoint generation, including
                // its checkpoint-owned model/anchor. Only host monotonic metadata
                // may be raised before recovery falls back to it.
                previous_checkpoint.loads_used =
                    previous_checkpoint.loads_used.max(checkpoint.loads_used);
                previous_checkpoint.run_peak_certified_power = previous_checkpoint
                    .run_peak_certified_power
                    .max(checkpoint.run_peak_certified_power);
                let backup_bytes = ActiveRunEnvelope::encode(&previous_checkpoint)?;
                ActiveRunEnvelope::decode(&backup_bytes)?;
                self.write_synced(BACKUP_TEMP_FILE, &backup_bytes)?;
                replace_file(&self.path(BACKUP_TEMP_FILE), &self.path(BACKUP_FILE))?;
            }
        }

        replace_file(&self.path(TEMP_FILE), &self.path(CURRENT_FILE))?;
        if replaces_invalidated_run {
            // A terminally invalidated generation must never become the rolling
            // backup of the next run, even if old files survived a prior cleanup.
            for file in [BACKUP_FILE, BACKUP_TEMP_FILE] {
                let path = self.path(file);
                if path.exists() {
                    fs::remove_file(path)?;
                }
            }
            fs::remove_file(invalidated)?;
        }
        self.sync_root()
    }

    pub fn invalidate(&self) -> Result<(), ActiveRunStoreError> {
        self.write_synced(INVALIDATED_FILE, b"invalidated\n")?;
        self.sync_root()?;
        for file in [CURRENT_FILE, BACKUP_FILE, TEMP_FILE, BACKUP_TEMP_FILE] {
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
    ) -> Result<Option<ActiveRunCheckpointExport>, ActiveRunStoreError> {
        let path = self.path(file_name);
        if !path.exists() {
            return Ok(None);
        }
        let mut bytes = Vec::new();
        File::open(path)?.read_to_end(&mut bytes)?;
        ActiveRunEnvelope::decode(&bytes).map(Some)
    }

    fn restore_backup_to_current(&self) -> Result<(), ActiveRunStoreError> {
        let bytes = fs::read(self.path(BACKUP_FILE))?;
        ActiveRunEnvelope::decode(&bytes)?;
        self.write_synced(TEMP_FILE, &bytes)?;
        replace_file(&self.path(TEMP_FILE), &self.path(CURRENT_FILE))?;
        self.sync_root()
    }

    fn write_synced(&self, file_name: &str, bytes: &[u8]) -> Result<(), ActiveRunStoreError> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(self.path(file_name))?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }

    fn sync_root(&self) -> Result<(), ActiveRunStoreError> {
        sync_directory(&self.root).map_err(ActiveRunStoreError::from)
    }

    fn path(&self, file_name: &str) -> PathBuf {
        self.root.join(file_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(payload: &[u8], loads_used: u8) -> ActiveRunCheckpointExport {
        ActiveRunCheckpointExport {
            core_payload_json: payload.to_vec(),
            loads_used,
            run_peak_certified_power: 100,
        }
    }

    #[test]
    fn publish_load_backup_recovery_and_invalidation_use_real_files() {
        let directory = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(directory.path()).unwrap();
        let first = checkpoint(br#"{"checkpoint":"first"}"#, 1);
        let second = checkpoint(br#"{"checkpoint":"second"}"#, 2);
        store.publish(&first).unwrap();
        store.publish(&second).unwrap();

        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
        assert_eq!(
            store.load().unwrap(),
            ActiveRunLoad::Loaded {
                checkpoint: checkpoint(br#"{"checkpoint":"first"}"#, 2),
                recovered_from_backup: true,
            }
        );
        assert_eq!(
            store.load().unwrap(),
            ActiveRunLoad::Loaded {
                checkpoint: checkpoint(br#"{"checkpoint":"first"}"#, 2),
                recovered_from_backup: false,
            }
        );

        store.invalidate().unwrap();
        assert_eq!(store.load().unwrap(), ActiveRunLoad::Missing);
    }

    #[test]
    fn backup_recovery_never_rolls_back_monotonic_load_count() {
        let directory = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(directory.path()).unwrap();
        store.publish(&checkpoint(b"save-point-a", 0)).unwrap();
        store.publish(&checkpoint(b"save-point-b", 1)).unwrap();
        store.publish(&checkpoint(b"save-point-b", 2)).unwrap();

        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
        assert_eq!(
            store.load().unwrap(),
            ActiveRunLoad::Loaded {
                checkpoint: checkpoint(b"save-point-b", 2),
                recovered_from_backup: true,
            }
        );
    }

    #[test]
    fn backup_recovery_preserves_checkpoint_payload_and_monotonic_power_peak() {
        let directory = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(directory.path()).unwrap();
        let first = checkpoint(b"save-point-a-anchor-100", 0);
        let mut second = checkpoint(b"save-point-b-anchor-375", 1);
        second.run_peak_certified_power = 450;
        store.publish(&first).unwrap();
        store.publish(&second).unwrap();

        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
        let ActiveRunLoad::Loaded {
            checkpoint,
            recovered_from_backup,
        } = store.load().unwrap()
        else {
            panic!("backup should recover the active run");
        };
        assert!(recovered_from_backup);
        assert_eq!(checkpoint.core_payload_json, b"save-point-a-anchor-100");
        assert_eq!(checkpoint.loads_used, 1);
        assert_eq!(checkpoint.run_peak_certified_power, 450);
    }

    #[test]
    fn publishing_after_terminal_invalidation_never_reuses_an_old_run_backup() {
        let directory = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(directory.path()).unwrap();
        store.publish(&checkpoint(b"old-run-a", 0)).unwrap();
        store.publish(&checkpoint(b"old-run-b", 0)).unwrap();
        let stale_backup = fs::read(directory.path().join(BACKUP_FILE)).unwrap();
        store.invalidate().unwrap();
        fs::write(directory.path().join(BACKUP_FILE), stale_backup).unwrap();

        store.publish(&checkpoint(b"new-run", 0)).unwrap();
        assert!(!directory.path().join(BACKUP_FILE).exists());
        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
        assert!(matches!(
            store.load(),
            Err(ActiveRunStoreError::BothCopiesInvalid { .. })
        ));
    }

    #[test]
    fn unsupported_version_both_corrupt_and_single_writer_are_explicit() {
        let directory = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(directory.path()).unwrap();
        store.publish(&checkpoint(b"payload", 0)).unwrap();

        let mut envelope: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.path().join(CURRENT_FILE)).unwrap())
                .unwrap();
        envelope["version"] = serde_json::json!(99);
        fs::write(
            directory.path().join(CURRENT_FILE),
            serde_json::to_vec(&envelope).unwrap(),
        )
        .unwrap();
        fs::write(directory.path().join(BACKUP_FILE), b"also corrupt").unwrap();
        assert!(matches!(
            store.load(),
            Err(ActiveRunStoreError::BothCopiesInvalid { .. })
        ));
        assert!(matches!(
            ActiveRunStore::open(directory.path()),
            Err(ActiveRunStoreError::LockUnavailable(_))
        ));
    }

    #[test]
    fn checksum_mismatch_is_rejected() {
        let bytes = ActiveRunEnvelope::encode(&checkpoint(b"payload", 0)).unwrap();
        let mut envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        envelope["monotonic"]["loads_used"] = serde_json::json!(2);
        let bytes = serde_json::to_vec(&envelope).unwrap();
        assert!(matches!(
            ActiveRunEnvelope::decode(&bytes),
            Err(ActiveRunStoreError::ChecksumMismatch)
        ));
    }

    #[test]
    fn publish_failure_keeps_previous_current_readable() {
        let directory = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(directory.path()).unwrap();
        let previous = checkpoint(b"previous", 1);
        store.publish(&previous).unwrap();
        fs::create_dir(directory.path().join(TEMP_FILE)).unwrap();

        assert!(store.publish(&checkpoint(b"next", 2)).is_err());
        assert_eq!(
            store.load().unwrap(),
            ActiveRunLoad::Loaded {
                checkpoint: previous,
                recovered_from_backup: false,
            }
        );
    }

    #[test]
    fn unsupported_envelope_version_is_not_silently_migrated() {
        let bytes = ActiveRunEnvelope::encode(&checkpoint(b"payload", 0)).unwrap();
        let mut envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        envelope["version"] = serde_json::json!(99);
        assert!(matches!(
            ActiveRunEnvelope::decode(&serde_json::to_vec(&envelope).unwrap()),
            Err(ActiveRunStoreError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn old_envelope_version_and_removed_monotonic_anchor_are_rejected() {
        let bytes = ActiveRunEnvelope::encode(&checkpoint(b"payload", 0)).unwrap();
        let current: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(current["version"], 3);
        assert!(current["monotonic"].get("power_model_version").is_none());
        assert!(current["monotonic"]
            .get("current_floor_power_anchor")
            .is_none());

        let mut old_version = current.clone();
        old_version["version"] = serde_json::json!(2);
        assert!(matches!(
            ActiveRunEnvelope::decode(&serde_json::to_vec(&old_version).unwrap()),
            Err(ActiveRunStoreError::UnsupportedVersion(2))
        ));

        let mut removed_field = current;
        removed_field["monotonic"]["current_floor_power_anchor"] = serde_json::json!(100);
        assert!(matches!(
            ActiveRunEnvelope::decode(&serde_json::to_vec(&removed_field).unwrap()),
            Err(ActiveRunStoreError::InvalidEnvelope(_))
        ));
    }
}
