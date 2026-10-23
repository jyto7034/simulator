use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use fs2::FileExt;
use game_core::application::ApplicationCheckpoint;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{replace_file, sync_directory};
use super::mod_requirements::ApprovedStoreGate;
use game_core::mod_validation::RequiredConfiguration;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredCheckpoint {
    pub core_payload_json: Vec<u8>,
    pub loads_used: u8,
    pub run_peak_certified_power: u64,
}
impl From<&ApplicationCheckpoint> for StoredCheckpoint {
    fn from(checkpoint: &ApplicationCheckpoint) -> Self {
        Self {
            core_payload_json: checkpoint.core_payload_json().to_vec(),
            loads_used: checkpoint.loads_used(),
            run_peak_certified_power: checkpoint.run_peak_certified_power(),
        }
    }
}
impl StoredCheckpoint {
    pub fn to_application(&self) -> ApplicationCheckpoint {
        ApplicationCheckpoint::new(
            self.core_payload_json.clone(),
            self.loads_used,
            self.run_peak_certified_power,
        )
    }
}

pub type ActiveRunRevision = Uuid;
const ACTIVE_RUN_VERSION: u32 = 7;
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
    UnsupportedCoreVersion(u32),
    ChecksumMismatch,
    ProtectedOriginal(String),
    BothCopiesInvalid {
        current: String,
        backup: String,
    },
    LockUnavailable(std::io::Error),
    RevisionConflict {
        expected: ActiveRunRevision,
        actual: ActiveRunRevision,
    },
}
impl std::fmt::Display for ActiveRunStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtectedOriginal(reason) => write!(formatter, "protected active-run original: {reason}"),
            Self::Io(error) => write!(formatter, "active-run filesystem error: {error}"),
            Self::InvalidEnvelope(error) => {
                write!(formatter, "invalid active-run envelope: {error}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported active-run version {version}")
            }
            Self::UnsupportedCoreVersion(version) => {
                write!(formatter, "unsupported active-run Core version {version}")
            }
            Self::ChecksumMismatch => formatter.write_str("active-run checksum mismatch"),
            Self::BothCopiesInvalid { current, backup } => write!(
                formatter,
                "both active-run copies are invalid (current: {current}; backup: {backup})"
            ),
            Self::LockUnavailable(error) => write!(
                formatter,
                "active-run single-writer lock unavailable: {error}"
            ),
            Self::RevisionConflict { expected, actual } => write!(
                formatter,
                "active-run revision conflict (expected {expected}, actual {actual})"
            ),
        }
    }
}
impl std::error::Error for ActiveRunStoreError {}
impl From<std::io::Error> for ActiveRunStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl ActiveRunStoreError {
    pub fn is_corrupt(&self) -> bool {
        matches!(
            self,
            Self::InvalidEnvelope(_) | Self::ChecksumMismatch | Self::BothCopiesInvalid { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveRunLoad {
    Missing,
    Loaded {
        checkpoint: StoredCheckpoint,
        recovered_from_backup: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActiveRunRecord {
    pub revision: ActiveRunRevision,
    pub checkpoint: Option<StoredCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_configuration: Option<RequiredConfiguration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveRunSnapshot {
    pub revision: ActiveRunRevision,
    pub checkpoint: Option<StoredCheckpoint>,
    pub recovered_from_backup: bool,
    pub required_configuration: Option<RequiredConfiguration>,
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
    revision: ActiveRunRevision,
    core_payload_base64: Option<String>,
    monotonic: Option<ActiveRunMonotonicMetadata>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActiveRunEnvelope {
    version: u32,
    revision: ActiveRunRevision,
    core_payload_base64: Option<String>,
    monotonic: Option<ActiveRunMonotonicMetadata>,
    checksum_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnsignedActiveRunEnvelopeV6 {
    version: u32,
    revision: ActiveRunRevision,
    core_payload_base64: Option<String>,
    monotonic: Option<ActiveRunMonotonicMetadata>,
    required_configuration: RequiredConfiguration,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActiveRunEnvelopeV6 {
    #[serde(flatten)]
    unsigned: UnsignedActiveRunEnvelopeV6,
    checksum_sha256: String,
}

fn core_version(bytes: &[u8]) -> Result<(), ActiveRunStoreError> {
    #[derive(Deserialize)]
    struct Header {
        version: u32,
    }
    let header: Header = serde_json::from_slice(bytes)
        .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
    // The envelope preserves prior opaque writers; V16 adds paid investment and warehouse consumption.
    // Core decides whether a payload version can be resumed.
    // V13 was an independent checkpoint format, never an active-run writer.
    if !matches!(header.version, 12 | 14 | 15 | 16 | 17) {
        return Err(ActiveRunStoreError::UnsupportedCoreVersion(header.version));
    }
    Ok(())
}

fn admit_active_output(bytes: Vec<u8>) -> Result<Vec<u8>, ActiveRunStoreError> {
    if bytes.len() > super::mod_requirements::ACTIVE_MAX_BYTES { return Err(ActiveRunStoreError::InvalidEnvelope("active envelope exceeds supported byte ceiling".into())); }
    Ok(bytes)
}

impl ActiveRunEnvelope {
    fn encode_record(record: &ActiveRunRecord) -> Result<Vec<u8>, ActiveRunStoreError> {
        if record.revision.is_nil() {
            return Err(ActiveRunStoreError::InvalidEnvelope(
                "nil publication revision".into(),
            ));
        }
        if let Some(checkpoint) = &record.checkpoint {
            core_version(&checkpoint.core_payload_json)?;
        }
        if let Some(required) = &record.required_configuration {
            validate_requirement(record)?;
            let unsigned = UnsignedActiveRunEnvelopeV6 {
                version: ACTIVE_RUN_VERSION, revision: record.revision,
                core_payload_base64: record.checkpoint.as_ref().map(|c| BASE64.encode(&c.core_payload_json)),
                monotonic: record.checkpoint.as_ref().map(|c| ActiveRunMonotonicMetadata { loads_used: c.loads_used, run_peak_certified_power: c.run_peak_certified_power }),
                required_configuration: required.clone(),
            };
            let checksum_sha256 = format!("{:x}", Sha256::digest(serde_json::to_vec(&unsigned).map_err(|e| ActiveRunStoreError::InvalidEnvelope(e.to_string()))?));
            let bytes = serde_json::to_vec_pretty(&ActiveRunEnvelopeV6 { unsigned, checksum_sha256 }).map_err(|e| ActiveRunStoreError::InvalidEnvelope(e.to_string()))?;
            return admit_active_output(bytes);
        }
        let unsigned = UnsignedActiveRunEnvelope {
            version: ACTIVE_RUN_VERSION,
            revision: record.revision,
            core_payload_base64: record
                .checkpoint
                .as_ref()
                .map(|checkpoint| BASE64.encode(&checkpoint.core_payload_json)),
            monotonic: record
                .checkpoint
                .as_ref()
                .map(|checkpoint| ActiveRunMonotonicMetadata {
                    loads_used: checkpoint.loads_used,
                    run_peak_certified_power: checkpoint.run_peak_certified_power,
                }),
        };
        let unsigned_bytes = serde_json::to_vec(&unsigned)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
        let envelope = Self {
            version: unsigned.version,
            revision: unsigned.revision,
            core_payload_base64: unsigned.core_payload_base64,
            monotonic: unsigned.monotonic,
            checksum_sha256: format!("{:x}", Sha256::digest(unsigned_bytes)),
        };
        admit_active_output(serde_json::to_vec_pretty(&envelope).map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?)
    }

    fn decode_record(bytes: &[u8]) -> Result<ActiveRunRecord, ActiveRunStoreError> {
        if bytes.len() > super::mod_requirements::ACTIVE_MAX_BYTES { return Err(ActiveRunStoreError::InvalidEnvelope("active envelope exceeds supported byte ceiling".into())); }
        #[derive(Deserialize)]
        struct Header {
            version: u32,
        }
        let header: Header = serde_json::from_slice(bytes)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
        if header.version != ACTIVE_RUN_VERSION { return Err(ActiveRunStoreError::UnsupportedVersion(header.version)); }
        if serde_json::from_slice::<serde_json::Value>(bytes).map_err(|e| ActiveRunStoreError::InvalidEnvelope(e.to_string()))?.get("required_configuration").is_some() {
            let envelope: ActiveRunEnvelopeV6 = serde_json::from_slice(bytes).map_err(|e| ActiveRunStoreError::InvalidEnvelope(e.to_string()))?;
            let unsigned = envelope.unsigned;
            if envelope.checksum_sha256 != format!("{:x}", Sha256::digest(serde_json::to_vec(&unsigned).map_err(|e| ActiveRunStoreError::InvalidEnvelope(e.to_string()))?)) {
                return Err(ActiveRunStoreError::ChecksumMismatch);
            }
            if unsigned.revision.is_nil() { return Err(ActiveRunStoreError::InvalidEnvelope("nil publication revision".into())); }
            let checkpoint = decode_checkpoint(unsigned.core_payload_base64, unsigned.monotonic)?;
            let record = ActiveRunRecord { revision: unsigned.revision, checkpoint, required_configuration: Some(unsigned.required_configuration) };
            validate_requirement(&record)?;
            return Ok(record);
        }
        if header.version != ACTIVE_RUN_VERSION {
            return Err(ActiveRunStoreError::UnsupportedVersion(header.version));
        }
        let envelope: Self = serde_json::from_slice(bytes)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
        let unsigned = UnsignedActiveRunEnvelope {
            version: envelope.version,
            revision: envelope.revision,
            core_payload_base64: envelope.core_payload_base64,
            monotonic: envelope.monotonic,
        };
        let unsigned_bytes = serde_json::to_vec(&unsigned)
            .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
        if envelope.checksum_sha256 != format!("{:x}", Sha256::digest(unsigned_bytes)) {
            return Err(ActiveRunStoreError::ChecksumMismatch);
        }
        if unsigned.revision.is_nil() {
            return Err(ActiveRunStoreError::InvalidEnvelope(
                "nil publication revision".into(),
            ));
        }
        let checkpoint = match (unsigned.core_payload_base64, unsigned.monotonic) {
            (None, None) => None,
            (Some(bytes), Some(monotonic)) if monotonic.loads_used <= 3 => {
                let core_payload_json = BASE64
                    .decode(bytes)
                    .map_err(|error| ActiveRunStoreError::InvalidEnvelope(error.to_string()))?;
                core_version(&core_payload_json)?;
                Some(StoredCheckpoint {
                    core_payload_json,
                    loads_used: monotonic.loads_used,
                    run_peak_certified_power: monotonic.run_peak_certified_power,
                })
            }
            _ => {
                return Err(ActiveRunStoreError::InvalidEnvelope(
                    "incoherent tombstone or load count".into(),
                ))
            }
        };
        Ok(ActiveRunRecord {
            revision: unsigned.revision,
            checkpoint,
            required_configuration: None,
        })
    }

    #[cfg(test)]
    fn encode(checkpoint: &StoredCheckpoint) -> Result<Vec<u8>, ActiveRunStoreError> {
        Self::encode_record(&ActiveRunRecord {
            revision: Uuid::new_v4(),
            checkpoint: Some(checkpoint.clone()),
            required_configuration: None,
        })
    }
    #[cfg(test)]
    fn decode(bytes: &[u8]) -> Result<StoredCheckpoint, ActiveRunStoreError> {
        Self::decode_record(bytes)?
            .checkpoint
            .ok_or_else(|| ActiveRunStoreError::InvalidEnvelope("tombstone".into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalFileIdentity { length: u64, sha256: String }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CorruptActiveOriginals {
    #[serde(deserialize_with = "required_original_option")]
    current: Option<OriginalFileIdentity>,
    #[serde(deserialize_with = "required_original_option")]
    backup: Option<OriginalFileIdentity>,
}
fn required_original_option<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where D: serde::Deserializer<'de>, T: Deserialize<'de> { Option::<T>::deserialize(d) }
fn original_identity(bytes: &[u8]) -> OriginalFileIdentity {
    OriginalFileIdentity { length: bytes.len() as u64, sha256: format!("{:x}", Sha256::digest(bytes)) }
}

pub struct ActiveRunStore {
    root: PathBuf,
    _lock: File,
    gate: Option<Arc<ApprovedStoreGate>>,
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
        Ok(Self { root, _lock: lock, gate: None })
    }

    pub(crate) fn open_approved(root: impl AsRef<Path>, gate: Arc<ApprovedStoreGate>) -> Result<Self, ActiveRunStoreError> {
        gate.preflight()?;
        let mut store = Self::open(root)?;
        store.gate = Some(gate);
        store.protect_versions()?;
        Ok(store)
    }

    /// Unsupported current, backup and legacy marker bytes are protected before
    /// recovery or mutation; a supported backup cannot hide an incompatible file.
    pub(crate) fn protect_versions(&self) -> Result<(), ActiveRunStoreError> {
        if let Some(gate) = &self.gate { gate.preflight()?; }
        if self.path(INVALIDATED_FILE).exists() {
            return Err(ActiveRunStoreError::UnsupportedVersion(4));
        }
        for file in [CURRENT_FILE, BACKUP_FILE] {
            match self.read_decoded(file) {
                Err(
                    error @ (ActiveRunStoreError::UnsupportedVersion(_)
                    | ActiveRunStoreError::UnsupportedCoreVersion(_) | ActiveRunStoreError::ProtectedOriginal(_)),
                ) => return Err(error),
                Err(ActiveRunStoreError::Io(error)) => return Err(ActiveRunStoreError::Io(error)),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Result<ActiveRunSnapshot, ActiveRunStoreError> {
        self.protect_versions()?;
        let (record, recovered_from_backup) = match self.read_decoded(CURRENT_FILE) {
            Ok(Some(record)) => (record, false),
            current => match self.read_decoded(BACKUP_FILE) {
                Ok(Some(record)) => {
                    let bytes = fs::read(self.path(BACKUP_FILE))?;
                    self.write_synced(TEMP_FILE, &bytes)?;
                    replace_file(&self.path(TEMP_FILE), &self.path(CURRENT_FILE))?;
                    self.sync_root()?;
                    (record, true)
                }
                Ok(None) => match current {
                    Ok(None) => (
                        ActiveRunRecord {
                            revision: Uuid::nil(),
                            checkpoint: None,
                            required_configuration: None,
                        },
                        false,
                    ),
                    Err(error) => {
                        return Err(ActiveRunStoreError::BothCopiesInvalid {
                            current: error.to_string(),
                            backup: "missing".into(),
                        })
                    }
                    Ok(Some(_)) => unreachable!(),
                },
                Err(error) => {
                    return Err(ActiveRunStoreError::BothCopiesInvalid {
                        current: current
                            .err()
                            .map_or_else(|| "missing".into(), |error| error.to_string()),
                        backup: error.to_string(),
                    })
                }
            },
        };
        Ok(ActiveRunSnapshot {
            revision: record.revision,
            checkpoint: record.checkpoint,
            recovered_from_backup,
            required_configuration: record.required_configuration,
        })
    }

    pub fn load(&self) -> Result<ActiveRunLoad, ActiveRunStoreError> {
        let snapshot = self.snapshot()?;
        Ok(match snapshot.checkpoint {
            Some(checkpoint) => ActiveRunLoad::Loaded {
                checkpoint,
                recovered_from_backup: snapshot.recovered_from_backup,
            },
            None => ActiveRunLoad::Missing,
        })
    }

    pub(crate) fn prepare_record(
        &self,
        expected: ActiveRunRevision,
        checkpoint: Option<&StoredCheckpoint>,
    ) -> Result<ActiveRunRecord, ActiveRunStoreError> {
        let current = self.snapshot()?;
        if current.revision != expected {
            return Err(ActiveRunStoreError::RevisionConflict {
                expected,
                actual: current.revision,
            });
        }
        let mut checkpoint = checkpoint.cloned();
        if let (Some(incoming), Some(previous)) = (&mut checkpoint, &current.checkpoint) {
            incoming.loads_used = incoming.loads_used.max(previous.loads_used);
            incoming.run_peak_certified_power = incoming
                .run_peak_certified_power
                .max(previous.run_peak_certified_power);
        }
        let required_configuration = match &self.gate { Some(gate) => Some(gate.configuration_for_checkpoint(checkpoint.as_ref(), current.required_configuration.as_ref())?), None => None };
        let record = ActiveRunRecord {
            revision: Uuid::new_v4(),
            checkpoint,
            required_configuration,
        };
        let bytes = ActiveRunEnvelope::encode_record(&record)?;
        ActiveRunEnvelope::decode_record(&bytes)?;
        Ok(record)
    }

    pub(crate) fn validate_record(record: &ActiveRunRecord) -> Result<(), ActiveRunStoreError> {
        ActiveRunEnvelope::decode_record(&ActiveRunEnvelope::encode_record(record)?)?;
        Ok(())
    }

    pub(crate) fn bind_record(&self, record: &mut ActiveRunRecord, required: &RequiredConfiguration) -> Result<(), ActiveRunStoreError> {
        let gate = self.gate.as_ref().ok_or_else(|| ActiveRunStoreError::ProtectedOriginal("approved store gate is missing".into()))?;
        gate.ensure_allowed(required)?;
        record.required_configuration = Some(required.clone());
        Self::validate_record(record)
    }

    pub(crate) fn validate_install(
        &self,
        expected: ActiveRunRevision,
        record: &ActiveRunRecord,
    ) -> Result<(), ActiveRunStoreError> {
        Self::validate_record(record)?;
        let current = self.snapshot()?;
        if current.revision == record.revision {
            if current.checkpoint != record.checkpoint || current.required_configuration != record.required_configuration {
                return Err(ActiveRunStoreError::InvalidEnvelope(
                    "published revision has different contents".into(),
                ));
            }
        } else if current.revision != expected {
            return Err(ActiveRunStoreError::RevisionConflict {
                expected,
                actual: current.revision,
            });
        }
        Ok(())
    }

    pub(crate) fn install_record(
        &self,
        expected: ActiveRunRevision,
        record: &ActiveRunRecord,
    ) -> Result<(), ActiveRunStoreError> {
        let current = self.snapshot()?;
        if current.revision == record.revision {
            if current.checkpoint != record.checkpoint || current.required_configuration != record.required_configuration {
                return Err(ActiveRunStoreError::InvalidEnvelope(
                    "published revision has different contents".into(),
                ));
            }
        } else if current.revision != expected {
            return Err(ActiveRunStoreError::RevisionConflict {
                expected,
                actual: current.revision,
            });
        }
        self.write_record(record)
    }

    fn write_record(&self, record: &ActiveRunRecord) -> Result<(), ActiveRunStoreError> {
        self.protect_versions()?;
        let bytes = ActiveRunEnvelope::encode_record(record)?;
        ActiveRunEnvelope::decode_record(&bytes)?;
        self.write_synced(TEMP_FILE, &bytes)?;
        super::durable_crash_point("active_temp_synced");
        replace_file(&self.path(TEMP_FILE), &self.path(CURRENT_FILE))?;
        super::durable_crash_point("active_renamed");
        self.sync_root()?;
        super::durable_crash_point("active_synced");
        self.write_synced(BACKUP_TEMP_FILE, &bytes)?;
        super::durable_crash_point("active_backup_temp_synced");
        replace_file(&self.path(BACKUP_TEMP_FILE), &self.path(BACKUP_FILE))?;
        self.sync_root()?;
        super::durable_crash_point("active_backup_synced");
        Ok(())
    }

    /// Administrative corruption reset preserves unsupported originals. The new
    /// tombstone has a unique revision, so an older missing token cannot return.
    /// Capture only supported, truly corrupt originals. No file repair or mutation occurs.
    pub(crate) fn prepare_corrupt_reset(&self) -> Result<(CorruptActiveOriginals, ActiveRunRecord), ActiveRunStoreError> {
        if self.path(INVALIDATED_FILE).exists() { return Err(ActiveRunStoreError::UnsupportedVersion(4)); }
        let current = super::mod_requirements::read_optional(&self.path(CURRENT_FILE), super::mod_requirements::ACTIVE_MAX_BYTES as u64)?;
        let backup = super::mod_requirements::read_optional(&self.path(BACKUP_FILE), super::mod_requirements::ACTIVE_MAX_BYTES as u64)?;
        if current.is_none() && backup.is_none() { return Err(ActiveRunStoreError::InvalidEnvelope("reset requires corrupt originals".into())); }
        for bytes in current.iter().chain(backup.iter()) {
            self.validate_corrupt_original(bytes)?;
            match ActiveRunEnvelope::decode_record(bytes) {
                Err(error) if error.is_corrupt() => {},
                Err(error) => return Err(error),
                Ok(_) => return Err(ActiveRunStoreError::InvalidEnvelope("reset cannot replace a valid recovery copy".into())),
            }
        }
        let originals = CorruptActiveOriginals { current: current.as_deref().map(original_identity), backup: backup.as_deref().map(original_identity) };
        let record = ActiveRunRecord { revision: Uuid::new_v4(), checkpoint: None,
            required_configuration: self.gate.as_ref().map(|gate| gate.configuration_for_checkpoint(None, None)).transpose()? };
        Self::validate_record(&record)?;
        Ok((originals, record))
    }

    fn validate_corrupt_original(&self, bytes: &[u8]) -> Result<(), ActiveRunStoreError> {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) {
            if let Some(version) = value.get("version").and_then(|value| value.as_u64()) {
                if version != u64::from(ACTIVE_RUN_VERSION) { return Err(ActiveRunStoreError::ProtectedOriginal(format!("active version {version}"))); }
            }
            if let Some(required) = value.get("required_configuration") {
                let required: RequiredConfiguration = serde_json::from_value(required.clone()).map_err(|e| ActiveRunStoreError::ProtectedOriginal(e.to_string()))?;
                required.validate().map_err(|e| ActiveRunStoreError::ProtectedOriginal(e.to_string()))?;
                if let Some(gate) = &self.gate { gate.ensure_allowed(&required)?; }
            }
            if let Some(encoded) = value.get("core_payload_base64").and_then(|value| value.as_str()) {
                if let Ok(core) = BASE64.decode(encoded) {
                    if let Err(error @ ActiveRunStoreError::UnsupportedCoreVersion(_)) = core_version(&core) { return Err(error); }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn verify_corrupt_reset_originals(root: &Path, originals: &CorruptActiveOriginals, next: &ActiveRunRecord) -> Result<(), ActiveRunStoreError> {
        let accepted = ActiveRunEnvelope::encode_record(next)?;
        for (name, expected) in [(CURRENT_FILE, &originals.current), (BACKUP_FILE, &originals.backup)] {
            let bytes = super::mod_requirements::read_optional(&root.join(name), super::mod_requirements::ACTIVE_MAX_BYTES as u64)?;
            let identity = bytes.as_deref().map(original_identity);
            // A completed per-file write from this same accepted intent is legal during recovery.
            if identity != *expected && bytes.as_deref() != Some(accepted.as_slice()) {
                return Err(ActiveRunStoreError::ProtectedOriginal(format!("corrupt reset original changed: {name}")));
            }
        }
        Ok(())
    }
    pub(crate) fn validate_corrupt_reset_install(&self, originals: &CorruptActiveOriginals, next: &ActiveRunRecord) -> Result<(), ActiveRunStoreError> {
        Self::validate_record(next)?;
        Self::verify_corrupt_reset_originals(&self.root, originals, next)
    }
    pub(crate) fn install_corrupt_reset(&self, originals: &CorruptActiveOriginals, next: &ActiveRunRecord) -> Result<(), ActiveRunStoreError> {
        self.validate_corrupt_reset_install(originals, next)?;
        self.write_record(next)
    }

    pub fn reset_corrupt(&self) -> Result<(), ActiveRunStoreError> {
        self.protect_versions()?;
        match self.snapshot() {
            Err(error) if error.is_corrupt() => self.write_record(&ActiveRunRecord {
                revision: Uuid::new_v4(),
                checkpoint: None,
                required_configuration: self.gate.as_ref().map(|g| g.configuration_for_checkpoint(None, None)).transpose()?,
            }),
            Err(error) => Err(error),
            Ok(_) => Err(ActiveRunStoreError::InvalidEnvelope(
                "reset requires corrupt data".into(),
            )),
        }
    }

    // Test fixture setup uses the same conditional install path as production.
    #[cfg(test)]
    pub fn publish(&self, checkpoint: &StoredCheckpoint) -> Result<(), ActiveRunStoreError> {
        let expected = self.snapshot()?.revision;
        let record = self.prepare_record(expected, Some(checkpoint))?;
        self.install_record(expected, &record)
    }
    #[cfg(test)]
    pub fn invalidate(&self) -> Result<(), ActiveRunStoreError> {
        let expected = self.snapshot()?.revision;
        let record = self.prepare_record(expected, None)?;
        self.install_record(expected, &record)
    }

    fn read_decoded(
        &self,
        file_name: &str,
    ) -> Result<Option<ActiveRunRecord>, ActiveRunStoreError> {
        let mut bytes = Vec::new();
        match File::open(self.path(file_name)) {
            Ok(mut file) => {
                if !file.metadata()?.is_file() || file.metadata()?.len() > super::mod_requirements::ACTIVE_MAX_BYTES as u64 { return Err(ActiveRunStoreError::ProtectedOriginal("unsupported file type or byte ceiling".into())); }
                file.take(super::mod_requirements::ACTIVE_MAX_BYTES as u64 + 1).read_to_end(&mut bytes)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }
        ActiveRunEnvelope::decode_record(&bytes).map(Some)
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
        sync_directory(&self.root).map_err(Into::into)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

fn decode_checkpoint(bytes: Option<String>, monotonic: Option<ActiveRunMonotonicMetadata>) -> Result<Option<StoredCheckpoint>, ActiveRunStoreError> {
    match (bytes, monotonic) {
        (None, None) => Ok(None),
        (Some(bytes), Some(m)) if m.loads_used <= 3 => {
            let core_payload_json = BASE64.decode(bytes).map_err(|e| ActiveRunStoreError::InvalidEnvelope(e.to_string()))?;
            core_version(&core_payload_json)?;
            Ok(Some(StoredCheckpoint { core_payload_json, loads_used: m.loads_used, run_peak_certified_power: m.run_peak_certified_power }))
        }
        _ => Err(ActiveRunStoreError::InvalidEnvelope("incoherent tombstone or load count".into())),
    }
}
fn validate_requirement(record: &ActiveRunRecord) -> Result<(), ActiveRunStoreError> {
    if let Some(required) = &record.required_configuration {
        required.validate().map_err(|e| ActiveRunStoreError::ProtectedOriginal(e.to_string()))?;
        if let Some(checkpoint) = &record.checkpoint {
            let identity = game_core::application::CoreApplication::inspect_checkpoint_requirements(&checkpoint.core_payload_json, 64 * 1024 * 1024).map_err(|e| ActiveRunStoreError::InvalidEnvelope(e.to_string()))?;
            if identity != required.content_identity { return Err(ActiveRunStoreError::ProtectedOriginal("Core/required configuration identity mismatch".into())); }
        }
    }
    Ok(())
}
impl ActiveRunStore {
    pub(crate) fn read_originals(root: &Path) -> Result<Vec<(String, Result<ActiveRunRecord, ActiveRunStoreError>)>, ActiveRunStoreError> {
        if root.join(INVALIDATED_FILE).exists() { return Err(ActiveRunStoreError::UnsupportedVersion(4)); }
        let mut records = Vec::new();
        for name in [CURRENT_FILE, BACKUP_FILE] {
            match super::mod_requirements::read_optional(&root.join(name), super::mod_requirements::ACTIVE_MAX_BYTES as u64)? {
                Some(bytes) => records.push((name.into(), ActiveRunEnvelope::decode_record(&bytes))),
                None => {},
            }
        }
        Ok(records)
    }
    pub(crate) fn readonly_record(&self) -> Result<ActiveRunRecord, ActiveRunStoreError> {
        let records = Self::read_originals(&self.root)?;
        for (_, record) in records { if let Ok(record) = record { return Ok(record); } }
        Ok(ActiveRunRecord { revision: Uuid::nil(), checkpoint: None, required_configuration: None })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u15_current_and_previous_core_writers_publish_and_reopen() {
        for version in [12, 14, 15, 16] {
            let directory = tempfile::tempdir().unwrap();
            let store = ActiveRunStore::open(directory.path()).unwrap();
            // This store owns the envelope, while Core owns the opaque payload.
            let checkpoint = StoredCheckpoint {
                core_payload_json: serde_json::to_vec(&serde_json::json!({"version":version,"fixture_payload":"opaque store payload"})).unwrap(),
                loads_used: 1,
                run_peak_certified_power: 100,
            };
            store.publish(&checkpoint).unwrap();
            drop(store);
            let store = ActiveRunStore::open(directory.path()).unwrap();
            assert!(matches!(store.load().unwrap(),ActiveRunLoad::Loaded {checkpoint: actual,..} if actual == checkpoint));
        }
    }

    fn checkpoint(payload: &[u8], loads_used: u8) -> StoredCheckpoint {
        StoredCheckpoint {
            core_payload_json: serde_json::to_vec(
                &serde_json::json!({"version":12,"fixture_payload":payload}),
            )
            .unwrap(),
            loads_used,
            run_peak_certified_power: 100,
        }
    }

    fn checkpoint_with_monotonic(
        payload: &[u8],
        loads_used: u8,
        run_peak_certified_power: u64,
    ) -> StoredCheckpoint {
        StoredCheckpoint {
            core_payload_json: serde_json::to_vec(
                &serde_json::json!({"version":12,"fixture_payload":payload}),
            )
            .unwrap(),
            loads_used,
            run_peak_certified_power,
        }
    }

    fn assert_publish_merges_current_monotonic_metadata(
        current_loads: u8,
        current_peak: u64,
        incoming_loads: u8,
        incoming_peak: u64,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(directory.path()).unwrap();
        let current = checkpoint_with_monotonic(b"higher-current", current_loads, current_peak);
        let incoming = checkpoint_with_monotonic(b"lower-incoming", incoming_loads, incoming_peak);
        let expected_loads = current_loads.max(incoming_loads);
        let expected_peak = current_peak.max(incoming_peak);

        store.publish(&current).unwrap();
        store.publish(&incoming).unwrap();
        drop(store);

        let reopened = ActiveRunStore::open(directory.path()).unwrap();
        assert_eq!(
            reopened.load().unwrap(),
            ActiveRunLoad::Loaded {
                checkpoint: checkpoint_with_monotonic(
                    b"lower-incoming",
                    expected_loads,
                    expected_peak,
                ),
                recovered_from_backup: false,
            },
            "installed current must keep incoming Core bytes and merged host metadata"
        );

        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
        assert_eq!(
            reopened.load().unwrap(),
            ActiveRunLoad::Loaded {
                checkpoint: checkpoint_with_monotonic(
                    b"lower-incoming",
                    expected_loads,
                    expected_peak,
                ),
                recovered_from_backup: true,
            },
            "backup must keep the latest confirmed Core bytes and the same host maxima"
        );
    }

    #[test]
    fn publish_merges_current_monotonic_load_count_into_incoming_generation() {
        assert_publish_merges_current_monotonic_metadata(3, 100, 1, 100);
    }

    #[test]
    fn publish_merges_current_monotonic_power_peak_into_incoming_generation() {
        assert_publish_merges_current_monotonic_metadata(1, 500, 1, 200);
    }

    #[test]
    fn publish_merges_current_monotonic_loads_and_peak_into_incoming_generation() {
        assert_publish_merges_current_monotonic_metadata(3, 500, 1, 200);
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
                checkpoint: checkpoint(br#"{"checkpoint":"second"}"#, 2),
                recovered_from_backup: true,
            }
        );
        assert_eq!(
            store.load().unwrap(),
            ActiveRunLoad::Loaded {
                checkpoint: checkpoint(br#"{"checkpoint":"second"}"#, 2),
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
        assert_eq!(checkpoint.core_payload_json, second.core_payload_json);
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
        assert!(directory.path().join(BACKUP_FILE).is_file());
        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
        assert_eq!(
            store.load().unwrap(),
            ActiveRunLoad::Loaded {
                checkpoint: checkpoint(b"new-run", 0),
                recovered_from_backup: true
            }
        );
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
            Err(ActiveRunStoreError::UnsupportedVersion(99))
        ));
        fs::write(directory.path().join(CURRENT_FILE), b"corrupt current").unwrap();
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
    fn previous_core_version_protects_current_and_backup_from_all_writes() {
        for old_slot in [CURRENT_FILE, BACKUP_FILE] {
            let directory = tempfile::tempdir().unwrap();
            let store = ActiveRunStore::open(directory.path()).unwrap();
            store.publish(&checkpoint(b"current", 0)).unwrap();
            let current_bytes = fs::read(directory.path().join(CURRENT_FILE)).unwrap();
            let mut envelope: ActiveRunEnvelope = serde_json::from_slice(&current_bytes).unwrap();
            envelope.core_payload_base64 =
                Some(BASE64.encode(br#"{"version":10,"original":true}"#));
            let unsigned = UnsignedActiveRunEnvelope {
                version: envelope.version,
                revision: envelope.revision,
                core_payload_base64: envelope.core_payload_base64.clone(),
                monotonic: envelope.monotonic.clone(),
            };
            envelope.checksum_sha256 = format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&unsigned).unwrap())
            );
            let original = serde_json::to_vec(&envelope).unwrap();
            fs::write(directory.path().join(old_slot), &original).unwrap();
            let before_current = fs::read(directory.path().join(CURRENT_FILE)).unwrap();
            let before_backup = fs::read(directory.path().join(BACKUP_FILE)).ok();
            assert!(matches!(
                store.protect_versions(),
                Err(ActiveRunStoreError::UnsupportedCoreVersion(10))
            ));
            assert!(matches!(
                store.publish(&checkpoint(b"new", 0)),
                Err(ActiveRunStoreError::UnsupportedCoreVersion(10))
            ));
            assert!(matches!(
                store.invalidate(),
                Err(ActiveRunStoreError::UnsupportedCoreVersion(10))
            ));
            assert_eq!(
                fs::read(directory.path().join(CURRENT_FILE)).unwrap(),
                before_current
            );
            assert_eq!(
                fs::read(directory.path().join(BACKUP_FILE)).ok(),
                before_backup
            );
            assert_eq!(fs::read(directory.path().join(old_slot)).unwrap(), original);
        }
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
        assert_eq!(current["version"], 7);
        assert!(current["monotonic"].get("power_model_version").is_none());
        assert!(current["monotonic"]
            .get("current_floor_power_anchor")
            .is_none());

        for old_version_number in [2, 3, 4, 5, 6] {
            let mut old_version = current.clone();
            old_version["version"] = serde_json::json!(old_version_number);
            assert!(matches!(
                ActiveRunEnvelope::decode(&serde_json::to_vec(&old_version).unwrap()),
                Err(ActiveRunStoreError::UnsupportedVersion(version)) if version == old_version_number
            ));
        }

        let mut removed_field = current;
        removed_field["monotonic"]["current_floor_power_anchor"] = serde_json::json!(100);
        assert!(matches!(
            ActiveRunEnvelope::decode(&serde_json::to_vec(&removed_field).unwrap()),
            Err(ActiveRunStoreError::InvalidEnvelope(_))
        ));
    }
}
