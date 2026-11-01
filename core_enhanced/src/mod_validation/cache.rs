//! Issuer-private authenticated accepted records. No player or worker can mint
//! these records; a cache record never contains a generation or executable token.
use super::{LabOutcome, ModValidationError, ModValidationErrorKind};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;
const MAX_RECORD_BYTES: u64 = 8 * 1024 * 1024;
const MAX_RECORDS: usize = 128;
const DOMAIN: &[u8] = b"CMVA accepted result v1\0";
const CRITERION_DOMAIN: &[u8] = b"CMVA adopted calibration v1\0";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdoptedCriterion {
    schema_version: u32,
    issuer: Uuid,
    context: String,
    criterion: super::lab::CriterionWire,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AcceptedRecord {
    pub schema_version: u32,
    pub issuer: Uuid,
    pub key: String,
    pub revision: u64,
    pub nonce: Uuid,
    pub result: LabOutcome,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload: Vec<u8>,
    tag: Vec<u8>,
}
pub(crate) struct CacheRoot {
    root: PathBuf,
    issuer: Uuid,
    secret: [u8; 32],
    lock: File,
    root_identity: (u64, u64),
    lock_identity: (u64, u64),
    key_identity: (u64, u64),
}
pub(crate) enum Publication {
    Durable,
    RecoveryPending(String),
}
impl CacheRoot {
    #[cfg(unix)]
    pub(crate) fn open(root: &Path) -> Result<Self, ModValidationError> {
        use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
        if !root.exists() {
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder.create(root)?;
        }
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o077 != 0
        {
            return Err(ModValidationError::incomplete(
                "cache.root",
                "issuer root must be an owned private directory",
            ));
        }
        let root = root.canonicalize()?;
        let nofollow = rustix::fs::OFlags::NOFOLLOW.bits() as i32;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(0o600)
            .custom_flags(nofollow)
            .open(root.join("issuer.lock"))?;
        private_file(&lock)?;
        fs2::FileExt::try_lock_exclusive(&lock)
            .map_err(|e| ModValidationError::incomplete("cache.lock", e.to_string()))?;
        let key_path = root.join("issuer.key");
        let (issuer, secret) = match OpenOptions::new()
            .read(true)
            .custom_flags(nofollow)
            .open(&key_path)
        {
            Ok(mut f) => {
                private_file(&f)?;
                let mut bytes = [0; 48];
                f.read_exact(&mut bytes)?;
                let mut extra = [0; 1];
                if f.read(&mut extra)? != 0 {
                    return Err(ModValidationError::incomplete(
                        "cache.key",
                        "invalid secret length",
                    ));
                }
                let issuer = Uuid::from_slice(&bytes[..16])
                    .map_err(|_| ModValidationError::incomplete("cache.key", "invalid issuer"))?;
                let mut secret = [0; 32];
                secret.copy_from_slice(&bytes[16..]);
                (issuer, secret)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let issuer = Uuid::new_v4();
                let mut secret = [0; 32];
                File::open("/dev/urandom")?.read_exact(&mut secret)?;
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .custom_flags(nofollow)
                    .open(&key_path)?;
                file.write_all(issuer.as_bytes())?;
                file.write_all(&secret)?;
                file.sync_all()?;
                File::open(&root)?.sync_all()?;
                (issuer, secret)
            }
            Err(e) => return Err(e.into()),
        };
        let root_identity = (metadata.dev(), metadata.ino());
        let lock_meta = lock.metadata()?;
        let key_meta = fs::symlink_metadata(&key_path)?;
        Ok(Self {
            root,
            issuer,
            secret,
            lock,
            root_identity,
            lock_identity: (lock_meta.dev(), lock_meta.ino()),
            key_identity: (key_meta.dev(), key_meta.ino()),
        })
    }
    #[cfg(not(unix))]
    pub(crate) fn open(_root: &Path) -> Result<Self, ModValidationError> {
        Err(ModValidationError::incomplete(
            "cache.root",
            "private no-follow issuer root unsupported on this platform",
        ))
    }
    pub(crate) fn issuer(&self) -> Uuid {
        self.issuer
    }
    fn mac(&self, payload: &[u8]) -> Hmac<Sha256> {
        self.mac_domain(DOMAIN, payload)
    }
    fn mac_domain(&self, domain: &[u8], payload: &[u8]) -> Hmac<Sha256> {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.secret).expect("fixed HMAC key length");
        mac.update(domain);
        mac.update(payload);
        mac
    }
    fn check_root(&self) -> Result<(), ModValidationError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let m = fs::symlink_metadata(&self.root)?;
            if !m.is_dir()
                || m.uid() != rustix::process::geteuid().as_raw()
                || m.mode() & 0o077 != 0
                || (m.dev(), m.ino()) != self.root_identity
            {
                return Err(ModValidationError::incomplete(
                    "cache.root",
                    "private root changed",
                ));
            }
            for (name, identity) in [
                ("issuer.lock", self.lock_identity),
                ("issuer.key", self.key_identity),
            ] {
                let file = open_private_read(&self.root.join(name))?;
                let metadata = file.metadata()?;
                if (metadata.dev(), metadata.ino()) != identity {
                    return Err(ModValidationError::incomplete(
                        "cache.owner",
                        "issuer lock/key replaced",
                    ));
                }
            }
            private_file(&self.lock)?;
        }
        Ok(())
    }
    fn records(&self) -> Result<Vec<PathBuf>, ModValidationError> {
        self.check_root()?;
        let mut accepted = Vec::new();
        let mut partial = Vec::new();
        let mut scanned = 0;
        for entry in fs::read_dir(&self.root)? {
            scanned += 1;
            if scanned > MAX_RECORDS * 2 + 16 {
                return Err(ModValidationError::incomplete(
                    "cache.directory",
                    "directory scan finite ceiling",
                ));
            }
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".accepted.json") {
                accepted.push(entry.path());
            } else if name.ends_with(".partial") {
                partial.push(entry.path());
            }
            if accepted.len() + partial.len() > MAX_RECORDS {
                return Err(ModValidationError::incomplete(
                    "cache.records",
                    "finite record admission exhausted",
                ));
            }
        }
        // Only temporary files created in this private root are cleanup candidates;
        // accepted records are never removed as cancellation or recovery cleanup.
        for path in partial {
            let m = fs::symlink_metadata(&path)?;
            if m.is_file() {
                fs::remove_file(path)?;
            }
        }
        accepted.sort();
        Ok(accepted)
    }
    fn read(&self, path: &Path) -> Result<AcceptedRecord, ModValidationError> {
        let file = open_private_read(path)?;
        let n = file.metadata()?.len();
        if n > MAX_RECORD_BYTES {
            return Err(ModValidationError::incomplete(
                "cache.record",
                "record byte ceiling",
            ));
        }
        let mut bytes = Vec::with_capacity(n as usize);
        file.take(MAX_RECORD_BYTES + 1).read_to_end(&mut bytes)?;
        let envelope: Envelope = serde_json::from_slice(&bytes).map_err(|_| {
            ModValidationError::incomplete("cache.record", "invalid accepted envelope")
        })?;
        self.mac(&envelope.payload)
            .verify_slice(&envelope.tag)
            .map_err(|_| {
                ModValidationError::new(
                    ModValidationErrorKind::ForeignIssuer,
                    "cache.record",
                    "authentication failed",
                )
            })?;
        let record: AcceptedRecord = serde_json::from_slice(&envelope.payload).map_err(|_| {
            ModValidationError::incomplete("cache.record", "invalid authenticated payload")
        })?;
        if record.schema_version != 1
            || record.issuer != self.issuer
            || !valid_key(&record.key)
            || record.revision == 0
            || !record.result.is_passed()
            || path.file_name().is_none_or(|n| {
                n.to_string_lossy() != format!("{}-{}.accepted.json", record.key, record.nonce)
            })
        {
            return Err(ModValidationError::incomplete(
                "cache.record",
                "not a complete exactly named accepted result",
            ));
        }
        Ok(record)
    }
    pub(crate) fn find(&self, key: &str) -> Result<Option<AcceptedRecord>, ModValidationError> {
        if !valid_key(key) {
            return Err(ModValidationError::invalid(
                "cache.key",
                "invalid exact key",
            ));
        }
        let mut latest: Option<AcceptedRecord> = None;
        for path in self.records()? {
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(&format!("{key}-")))
            {
                let record = self.read(&path)?;
                if record.key != key {
                    return Err(ModValidationError::incomplete(
                        "cache.key",
                        "authenticated key/lookup mismatch",
                    ));
                }
                if latest
                    .as_ref()
                    .is_none_or(|r| (r.revision, r.nonce) < (record.revision, record.nonce))
                {
                    latest = Some(record);
                }
            }
        }
        Ok(latest)
    }
    pub(crate) fn criterion(
        &self,
        context: &str,
    ) -> Result<Option<super::LabCriterion>, ModValidationError> {
        self.records()?;
        if !valid_key(context) {
            return Err(ModValidationError::invalid(
                "criterion.context",
                "invalid context",
            ));
        }
        let path = self.root.join(format!("{context}.criterion.json"));
        match fs::symlink_metadata(&path) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        let file = open_private_read(&path)?;
        if file.metadata()?.len() > MAX_RECORD_BYTES {
            return Err(ModValidationError::incomplete(
                "criterion.cache",
                "finite criterion evidence byte ceiling",
            ));
        }
        let mut bytes = Vec::new();
        file.take(MAX_RECORD_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_RECORD_BYTES {
            return Err(ModValidationError::incomplete(
                "criterion.cache",
                "criterion grew past finite byte ceiling",
            ));
        }
        let envelope: Envelope = serde_json::from_slice(&bytes).map_err(|_| {
            ModValidationError::incomplete("criterion.cache", "invalid calibration envelope")
        })?;
        self.mac_domain(CRITERION_DOMAIN, &envelope.payload)
            .verify_slice(&envelope.tag)
            .map_err(|_| {
                ModValidationError::new(
                    ModValidationErrorKind::ForeignIssuer,
                    "criterion.cache",
                    "authentication failed",
                )
            })?;
        let record: AdoptedCriterion = serde_json::from_slice(&envelope.payload).map_err(|_| {
            ModValidationError::incomplete("criterion.cache", "invalid authenticated calibration")
        })?;
        if record.schema_version != 1 || record.issuer != self.issuer || record.context != context {
            return Err(ModValidationError::incomplete(
                "criterion.cache",
                "issuer/context mismatch",
            ));
        }
        let criterion = super::LabCriterion::restore(record.criterion, context)?;
        open_private_read(&path)?.sync_all()?;
        File::open(&self.root)?.sync_all()?;
        Ok(Some(criterion))
    }
    pub(crate) fn adopt_criterion(
        &self,
        context: &str,
        criterion: &super::LabCriterion,
    ) -> Result<(), ModValidationError> {
        if !criterion.matches_context(context) {
            return Err(ModValidationError::incomplete(
                "criterion.adoption",
                "actual current calibration required",
            ));
        }
        if let Some(previous) = self.criterion(context)? {
            if super::digest(&previous)? == super::digest(criterion)? {
                return Ok(());
            }
            return Err(ModValidationError::incomplete(
                "criterion.adoption",
                "adoption differs: use a new versioned profile context; prior criterion preserved",
            ));
        }
        let payload = serde_json::to_vec(&AdoptedCriterion {
            schema_version: 1,
            issuer: self.issuer,
            context: context.into(),
            criterion: criterion.wire(),
        })
        .map_err(|e| ModValidationError::incomplete("criterion.cache", e.to_string()))?;
        let tag = self
            .mac_domain(CRITERION_DOMAIN, &payload)
            .finalize()
            .into_bytes()
            .to_vec();
        let bytes = serde_json::to_vec(&Envelope { payload, tag })
            .map_err(|e| ModValidationError::incomplete("criterion.cache", e.to_string()))?;
        if bytes.len() as u64 > MAX_RECORD_BYTES {
            return Err(ModValidationError::incomplete(
                "criterion.cache",
                "finite criterion evidence byte ceiling",
            ));
        }
        let temporary = self.root.join(format!("{context}.partial"));
        let accepted = self.root.join(format!("{context}.criterion.json"));
        let before = (|| {
            let mut file = create_private_new(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            self.check_root()?;
            fs::rename(&temporary, &accepted)?;
            Ok::<(), ModValidationError>(())
        })();
        if let Err(e) = before {
            let _ = fs::remove_file(&temporary);
            return Err(e);
        }
        // An adopted criterion contains no generation approval. A post-rename
        // sync failure blocks issuer construction; restart authenticates and
        // re-confirms durability before restoring this same sealed criterion.
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
    pub(crate) fn publish(
        &self,
        record: &AcceptedRecord,
        before_accept: impl FnOnce() -> Result<(), ModValidationError>,
    ) -> Result<Publication, ModValidationError> {
        self.check_root()?;
        let records = self.records()?;
        if records.len() >= MAX_RECORDS
            || record.issuer != self.issuer
            || !record.result.is_passed()
            || !valid_key(&record.key)
        {
            return Err(ModValidationError::incomplete(
                "cache.publish",
                "invalid/finite accepted record",
            ));
        }
        let payload = serde_json::to_vec(record)
            .map_err(|e| ModValidationError::incomplete("cache.payload", e.to_string()))?;
        let tag = self.mac(&payload).finalize().into_bytes().to_vec();
        let bytes = serde_json::to_vec(&Envelope { payload, tag })
            .map_err(|e| ModValidationError::incomplete("cache.envelope", e.to_string()))?;
        if bytes.len() as u64 > MAX_RECORD_BYTES {
            return Err(ModValidationError::incomplete(
                "cache.record",
                "record byte ceiling",
            ));
        }
        let basename = format!("{}-{}", record.key, record.nonce);
        let temporary = self.root.join(format!("{basename}.partial"));
        let accepted = self.root.join(format!("{basename}.accepted.json"));
        let preaccept = (|| {
            let mut file = create_private_new(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            before_accept()?;
            if accepted.exists() {
                return Err(ModValidationError::incomplete(
                    "cache.publish",
                    "duplicate accepted nonce",
                ));
            }
            fs::rename(&temporary, &accepted)?;
            Ok::<(), ModValidationError>(())
        })();
        if let Err(e) = preaccept {
            let _ = fs::remove_file(&temporary);
            return Err(e);
        }
        // The accepted rename has happened. This cannot be relabelled cancelled
        // or incomplete, nor may an approval escape until durability is known.
        #[cfg(test)]
        if let Some(reason) = super::u11_cache_hooks::post_rename_fault() {
            return Ok(Publication::RecoveryPending(reason));
        }
        Ok(match File::open(&self.root).and_then(|f| f.sync_all()) {
            Ok(()) => Publication::Durable,
            Err(e) => Publication::RecoveryPending(e.to_string()),
        })
    }
    pub(crate) fn recover(&self, record: &AcceptedRecord) -> Result<(), ModValidationError> {
        self.check_root()?;
        let path = self
            .root
            .join(format!("{}-{}.accepted.json", record.key, record.nonce));
        let actual = self.read(&path)?;
        if actual.key != record.key
            || actual.nonce != record.nonce
            || actual.revision != record.revision
        {
            return Err(ModValidationError::incomplete(
                "cache.recover",
                "accepted record changed",
            ));
        }
        open_private_read(&path)?.sync_all()?;
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
}
fn valid_key(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
#[cfg(unix)]
fn private_file(file: &File) -> Result<(), ModValidationError> {
    use std::os::unix::fs::MetadataExt;
    let m = file.metadata()?;
    if !m.is_file() || m.uid() != rustix::process::geteuid().as_raw() || m.mode() & 0o077 != 0 {
        Err(ModValidationError::incomplete(
            "cache.file",
            "owned private regular file required",
        ))
    } else {
        Ok(())
    }
}
#[cfg(unix)]
fn open_private_read(path: &Path) -> Result<File, ModValidationError> {
    use std::os::unix::fs::OpenOptionsExt;
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    private_file(&f)?;
    Ok(f)
}
#[cfg(unix)]
fn create_private_new(path: &Path) -> Result<File, ModValidationError> {
    use std::os::unix::fs::OpenOptionsExt;
    Ok(OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?)
}
#[cfg(not(unix))]
fn open_private_read(_path: &Path) -> Result<File, ModValidationError> {
    Err(ModValidationError::incomplete(
        "cache.file",
        "unsupported private platform",
    ))
}
#[cfg(not(unix))]
fn create_private_new(_path: &Path) -> Result<File, ModValidationError> {
    Err(ModValidationError::incomplete(
        "cache.file",
        "unsupported private platform",
    ))
}
