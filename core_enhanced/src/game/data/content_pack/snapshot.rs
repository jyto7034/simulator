use super::super::live_ron::{parse_live_ron, LiveRonError, LiveRonPhase};
use super::{CatalogKind, ContentSelection, InputLimits, PackManifest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFileReceipt {
    pub pack_id: String,
    pub path: String,
    pub byte_count: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReceipt {
    pub files: Vec<SourceFileReceipt>,
}

impl SourceReceipt {
    /// Bind only an exact path captured by the owned snapshot. A selection or
    /// aggregate address has no single pack; never infer one from a directory.
    pub(super) fn bind_diagnostic(&self, mut error: LiveRonError) -> LiveRonError {
        if error.pack_id.is_none() {
            if let Some(file) = self.files.iter().find(|f| f.path == error.source_file) {
                error.pack_id = Some(file.pack_id.clone());
            }
        }
        for source in &mut error.related_sources {
            if source.pack_id.is_none() {
                if let Some(file) = self.files.iter().find(|f| f.path == source.source_file) {
                    source.pack_id = Some(file.pack_id.clone());
                }
            }
        }
        error
    }
}

#[derive(Clone)]
pub(super) struct PackSnapshot {
    pub manifest: PackManifest,
    pub manifest_text: String,
    pub manifest_source: String,
    pub catalogs: Vec<(super::CatalogKind, String, String)>,
}

#[derive(Clone)]
pub(super) struct SourceSnapshot {
    pub packs: Vec<PackSnapshot>,
    pub receipt: SourceReceipt,
}

fn error(source: &str, cause: impl Into<String>) -> LiveRonError {
    LiveRonError::validation(
        source,
        "content_snapshot",
        None::<String>,
        None::<String>,
        cause,
    )
    .with_phase(LiveRonPhase::Acquisition)
}

pub(super) fn relative_path(path: &Path) -> Result<String, LiveRonError> {
    let Some(value) = path.to_str() else {
        return Err(error("selection", "path must be UTF-8"));
    };
    if value.is_empty()
        || path.is_absolute()
        || value.contains(['\\', ':'])
        || value.chars().any(char::is_control)
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(error(
            value,
            "path must contain only normal relative components",
        ));
    }
    Ok(value.into())
}

fn validate_limits(limits: &InputLimits) -> Result<(), LiveRonError> {
    if limits.max_packs == 0
        || limits.max_files == 0
        || limits.max_file_bytes == 0
        || limits.max_total_bytes == 0
        || !(1..=128).contains(&limits.max_nesting)
        || limits.max_file_bytes.checked_add(1).is_none()
        || limits.value_expr.validate().is_err()
    {
        return Err(error(
            "host_limits",
            "invalid trusted input limits; parser nesting must be 1..=128 and value expression limits must be within hard ceilings",
        ));
    }
    Ok(())
}

/// ron 0.5 has no deserializer depth setting. Bound actual delimiters before
/// deserialization, while treating strings/characters and comments as opaque.
pub(super) fn check_nesting(source: &str, input: &str, limit: usize) -> Result<(), LiveRonError> {
    let bytes = input.as_bytes();
    let mut index = 0;
    let mut stack = Vec::new();
    let invalid = |at: usize, cause: &str| {
        LiveRonError::validation(
            source,
            "content_snapshot",
            None::<String>,
            Some(format!("byte[{at}]")),
            cause,
        )
        .with_phase(LiveRonPhase::Structure)
    };
    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let start = index;
                index += 2;
                let mut depth = 1;
                while depth != 0 {
                    if index >= bytes.len() {
                        return Err(invalid(start, "unterminated block comment"));
                    }
                    if bytes.get(index..index + 2) == Some(b"/*") {
                        depth += 1;
                        index += 2;
                    } else if bytes.get(index..index + 2) == Some(b"*/") {
                        depth -= 1;
                        index += 2;
                    } else {
                        index += 1;
                    }
                    if depth > limit {
                        return Err(invalid(index, "comment nesting limit exceeded"));
                    }
                }
            }
            b'"' | b'\'' => {
                let quote = bytes[index];
                let start = index;
                index += 1;
                loop {
                    let Some(&byte) = bytes.get(index) else {
                        return Err(invalid(start, "unterminated quoted value"));
                    };
                    index += 1;
                    if byte == quote {
                        break;
                    }
                    if byte == b'\\' {
                        if index == bytes.len() {
                            return Err(invalid(start, "unterminated quoted escape"));
                        }
                        index += 1;
                    }
                }
            }
            b'r' => {
                let start = index;
                let mut end = index + 1;
                while bytes.get(end) == Some(&b'#') {
                    end += 1;
                }
                if bytes.get(end) != Some(&b'"') {
                    index = end;
                    continue;
                }
                let hashes = end - index - 1;
                if hashes > limit {
                    return Err(invalid(start, "raw string delimiter limit exceeded"));
                }
                index = end + 1;
                loop {
                    let Some(&byte) = bytes.get(index) else {
                        return Err(invalid(start, "unterminated raw string"));
                    };
                    index += 1;
                    if byte == b'"'
                        && bytes
                            .get(index..index + hashes)
                            .is_some_and(|tail| tail.iter().all(|b| *b == b'#'))
                    {
                        index += hashes;
                        break;
                    }
                }
            }
            b'(' | b'[' | b'{' => {
                stack.push(bytes[index]);
                if stack.len() > limit {
                    return Err(invalid(index, "parser nesting limit exceeded"));
                }
                index += 1;
            }
            b')' | b']' | b'}' => {
                let expected = match bytes[index] {
                    b')' => b'(',
                    b']' => b'[',
                    _ => b'{',
                };
                if stack.pop() != Some(expected) {
                    return Err(invalid(index, "unmatched closing delimiter"));
                }
                index += 1;
            }
            _ => index += 1,
        }
    }
    if !stack.is_empty() {
        return Err(invalid(index, "unclosed delimiter"));
    }
    Ok(())
}

pub(super) fn acquire(
    root: &Path,
    selection: &ContentSelection,
    limits: &InputLimits,
) -> Result<SourceSnapshot, LiveRonError> {
    acquire_with_hook(root, selection, limits, || {})
}

fn acquire_with_hook(
    root: &Path,
    selection: &ContentSelection,
    limits: &InputLimits,
    acquired: impl FnOnce(),
) -> Result<SourceSnapshot, LiveRonError> {
    validate_limits(limits)?;
    if selection.packs.is_empty() || selection.packs.len() > limits.max_packs {
        return Err(error(
            "selection",
            "empty selection or pack count limit exceeded",
        ));
    }
    let root = platform::Root::open(root)?;
    let mut inventory = Inventory::default();
    let mut packs = Vec::new();
    let mut pack_ids = BTreeSet::new();
    let mut paths = selection
        .packs
        .iter()
        .map(|p| relative_path(p))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        let manifest_source = format!("{path}/pack.ron");
        let input = inventory.read(&root, &manifest_source, limits, None)?;
        let manifest: PackManifest = parse_live_ron(&manifest_source, "pack_manifest", &input)?;
        manifest.validate(&manifest_source)?;
        if !pack_ids.insert(manifest.pack_id.clone()) {
            return Err(error(&manifest_source, "duplicate pack ID").with_pack(&manifest.pack_id));
        }
        let mut catalogs = Vec::new();
        let mut files = manifest
            .files
            .iter()
            .map(|file| {
                Ok((
                    relative_path(&file.path).map_err(|error| {
                        error.with_pack(&manifest.pack_id).with_catalog(file.kind)
                    })?,
                    file.kind,
                ))
            })
            .collect::<Result<Vec<_>, LiveRonError>>()?;
        files.sort_by(|a, b| a.0.cmp(&b.0));
        for (file, kind) in files {
            let source = format!("{path}/{file}");
            let input = inventory
                .read(&root, &source, limits, Some(kind))
                .map_err(|error| error.with_pack(&manifest.pack_id).with_catalog(kind))?;
            catalogs.push((kind, source, input));
        }
        for record in inventory.records.iter_mut().rev().take(catalogs.len() + 1) {
            record.pack_id = manifest.pack_id.clone();
        }
        packs.push(PackSnapshot {
            manifest,
            manifest_text: input,
            manifest_source,
            catalogs,
        });
    }
    acquired();
    Ok(SourceSnapshot {
        packs,
        receipt: inventory.finish(&root, limits)?,
    })
}

pub(super) fn acquire_official(
    root: &Path,
    manifest: PackManifest,
    limits: &InputLimits,
) -> Result<SourceSnapshot, LiveRonError> {
    validate_limits(limits)?;
    manifest.validate("pack.ron")?;
    let root = platform::Root::open(root)?;
    let mut inventory = Inventory::default();
    let wrapper = inventory.read(&root, "data/bundle.ron", limits, None)?;
    super::super::live_ron::validate_live_ron_manifest(&wrapper)?;
    let mut catalogs = Vec::new();
    for file in &manifest.files {
        let path = relative_path(&file.path)?;
        let input = inventory.read(&root, &path, limits, Some(file.kind))?;
        catalogs.push((file.kind, path, input));
    }
    for record in &mut inventory.records {
        record.pack_id = manifest.pack_id.clone();
    }
    Ok(SourceSnapshot {
        packs: vec![PackSnapshot {
            manifest,
            manifest_text: String::new(),
            manifest_source: "pack.ron".into(),
            catalogs,
        }],
        receipt: inventory.finish(&root, limits)?,
    })
}

pub(super) fn write_new_pack(
    source_root: &Path,
    output: &Path,
    files: &[(String, String)],
) -> Result<PathBuf, LiveRonError> {
    platform::write_new_pack(source_root, output, files)
}

struct FileRecord {
    pack_id: String,
    catalog_kind: Option<CatalogKind>,
    path: String,
    bytes: String,
    identity: platform::Identity,
}

#[derive(Default)]
struct Inventory {
    paths: BTreeSet<String>,
    case_prefixes: BTreeMap<String, String>,
    file_ids: BTreeSet<(u64, u64)>,
    total_bytes: usize,
    records: Vec<FileRecord>,
}

impl Inventory {
    fn finish(
        self,
        root: &platform::Root,
        limits: &InputLimits,
    ) -> Result<SourceReceipt, LiveRonError> {
        // Reopen each path from the pinned root, so observed replacement or
        // directory symlink changes fail rather than trusting old descriptors.
        for record in &self.records {
            let bind_source = |error: LiveRonError| {
                let error = error.with_pack(&record.pack_id);
                match record.catalog_kind {
                    Some(kind) => error.with_catalog(kind),
                    None => error,
                }
            };
            let (bytes, identity) = root
                .read(&record.path, limits.max_file_bytes)
                .map_err(bind_source)?;
            if identity != record.identity || bytes != record.bytes {
                return Err(bind_source(error(
                    &record.path,
                    "file changed during snapshot acquisition",
                )));
            }
        }
        let mut files = self
            .records
            .into_iter()
            .map(|record| SourceFileReceipt {
                pack_id: record.pack_id,
                path: record.path,
                byte_count: record.bytes.len(),
                sha256: format!("{:x}", Sha256::digest(record.bytes.as_bytes())),
            })
            .collect::<Vec<_>>();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(SourceReceipt { files })
    }
    fn read(
        &mut self,
        root: &platform::Root,
        path: &str,
        limits: &InputLimits,
        catalog_kind: Option<CatalogKind>,
    ) -> Result<String, LiveRonError> {
        if self.records.len() >= limits.max_files {
            return Err(error(path, "file count limit exceeded"));
        }
        if !self.paths.insert(path.into()) {
            return Err(error(path, "duplicate inventory path"));
        }
        let mut prefix = String::new();
        for component in path.split('/') {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(component);
            if self
                .case_prefixes
                .insert(prefix.to_lowercase(), prefix.clone())
                .is_some_and(|previous| previous != prefix)
            {
                return Err(error(path, "case-colliding path component"));
            }
        }
        let (bytes, identity) = root.read(path, limits.max_file_bytes)?;
        if !self.file_ids.insert(identity.file_id()) {
            return Err(error(
                path,
                "duplicate file identity (hardlink or path alias)",
            ));
        }
        self.total_bytes = self
            .total_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| error(path, "total byte count overflow"))?;
        if self.total_bytes > limits.max_total_bytes {
            return Err(error(path, "aggregate byte limit exceeded"));
        }
        let authored_yaml = catalog_kind == Some(CatalogKind::Skills)
            && path.ends_with(".skill.yaml");
        // YAML comments, scalars and indentation are not RON lexical tokens.
        // Its event parser applies the host nesting and compiler source limits.
        // File/aggregate byte limits remain acquisition's responsibility; the
        // lowered RON still receives per-expression limits before publication.
        if !authored_yaml {
            check_nesting(path, &bytes, limits.max_nesting)?;
        }
        if catalog_kind == Some(CatalogKind::Skills) && !authored_yaml {
            crate::game::value_expr::validate_expression_input(
                &bytes,
                limits.value_expr.max_input_bytes,
                limits.max_nesting,
            )
            .map_err(|expression_error| {
                LiveRonError::validation(
                    path,
                    "content_snapshot",
                    None::<String>,
                    Some(format!("byte[{}]", expression_error.byte_offset)),
                    expression_error.cause,
                )
                .with_phase(LiveRonPhase::Structure)
                .with_code("value_expr_input_limit")
            })?;
        }
        self.records.push(FileRecord {
            pack_id: String::new(),
            catalog_kind,
            path: path.into(),
            bytes: bytes.clone(),
            identity,
        });
        Ok(bytes)
    }
}

#[cfg(unix)]
mod platform {
    use super::{error, LiveRonError};
    use rustix::fs::{mkdirat, open, openat, Mode, OFlags};
    use std::{
        collections::BTreeMap,
        fs::{File, Metadata},
        io::{Read, Write},
        os::unix::fs::MetadataExt,
        path::{Path, PathBuf},
    };

    #[derive(Debug, PartialEq, Eq)]
    pub(super) struct Identity {
        device: u64,
        inode: u64,
        len: u64,
        modified: (i64, i64),
        changed: (i64, i64),
    }
    impl Identity {
        fn of(metadata: &Metadata) -> Self {
            Self {
                device: metadata.dev(),
                inode: metadata.ino(),
                len: metadata.len(),
                modified: (metadata.mtime(), metadata.mtime_nsec()),
                changed: (metadata.ctime(), metadata.ctime_nsec()),
            }
        }
        pub(super) fn file_id(&self) -> (u64, u64) {
            (self.device, self.inode)
        }
    }

    pub(super) struct Root(File);
    impl Root {
        pub(super) fn open(path: &Path) -> Result<Self, LiveRonError> {
            open(
                path,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map(|fd| Self(fd.into()))
            .map_err(|cause| {
                error(
                    "trusted_root",
                    format!("cannot open trusted root directory: {cause}"),
                )
            })
        }
        pub(super) fn read(
            &self,
            path: &str,
            limit: usize,
        ) -> Result<(String, Identity), LiveRonError> {
            super::relative_path(Path::new(path))?;
            let mut directory = self
                .0
                .try_clone()
                .map_err(|cause| error(path, cause.to_string()))?;
            let mut components = path.split('/').peekable();
            let mut file = loop {
                let component = components
                    .next()
                    .expect("normal path has at least one component");
                let mut flags =
                    OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
                if components.peek().is_some() {
                    flags |= OFlags::DIRECTORY;
                }
                let fd = openat(&directory, component, flags, Mode::empty())
                    .map_err(|cause| error(path, format!("no-follow file open failed: {cause}")))?;
                let opened = File::from(fd);
                if components.peek().is_none() {
                    break opened;
                }
                directory = opened;
            };
            let metadata = file
                .metadata()
                .map_err(|cause| error(path, cause.to_string()))?;
            if !metadata.is_file() {
                return Err(error(path, "input is not a regular file"));
            }
            if metadata.len() > limit as u64 {
                return Err(error(path, "per-file byte limit exceeded"));
            }
            let before = Identity::of(&metadata);
            let mut bytes = Vec::new();
            (&mut file)
                .take((limit + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|cause| error(path, cause.to_string()))?;
            if bytes.len() > limit {
                return Err(error(path, "per-file byte limit exceeded"));
            }
            let after = Identity::of(
                &file
                    .metadata()
                    .map_err(|cause| error(path, cause.to_string()))?,
            );
            if before != after {
                return Err(error(path, "file changed while being read"));
            }
            let input = String::from_utf8(bytes)
                .map_err(|cause| error(path, format!("input is not UTF-8: {cause}")))?;
            Ok((input, after))
        }
    }

    pub(super) fn write_new_pack(
        source_root: &Path,
        output: &Path,
        files: &[(String, String)],
    ) -> Result<PathBuf, LiveRonError> {
        let source = std::fs::canonicalize(source_root)
            .map_err(|cause| error("source_root", cause.to_string()))?;
        let output_text = output
            .to_str()
            .ok_or_else(|| error("output", "output path must be UTF-8"))?;
        let leaf = output_text.rsplit('/').next().unwrap_or("");
        super::relative_path(Path::new(leaf))?;
        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent =
            std::fs::canonicalize(parent).map_err(|cause| error(output_text, cause.to_string()))?;
        let output = parent.join(leaf);
        if output.starts_with(&source) {
            return Err(error(output_text, "output must be outside the source tree"));
        }
        let parent_root = Root::open(&parent)?;
        // Atomic directory creation rejects every existing leaf, including an
        // empty directory or symlink. All subsequent writes use pinned handles.
        mkdirat(&parent_root.0, leaf, Mode::from_bits_truncate(0o700)).map_err(|cause| {
            error(
                output_text,
                format!("cannot exclusively create output directory: {cause}"),
            )
        })?;
        let result = (|| {
            let directory = File::from(
                openat(
                    &parent_root.0,
                    leaf,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(|cause| error(output_text, cause.to_string()))?,
            );
            let output_identity = Identity::of(
                &directory
                    .metadata()
                    .map_err(|cause| error(output_text, cause.to_string()))?,
            )
            .file_id();
            let mut directories = BTreeMap::from([(String::new(), directory)]);
            for (path, input) in files {
                super::relative_path(Path::new(path))?;
                let mut components = path.split('/').peekable();
                let mut prefix = String::new();
                while let Some(component) = components.next() {
                    let directory = directories
                        .get(&prefix)
                        .expect("parent created by this writer");
                    if components.peek().is_none() {
                        let fd = openat(
                            directory,
                            component,
                            OFlags::WRONLY
                                | OFlags::CREATE
                                | OFlags::EXCL
                                | OFlags::NOFOLLOW
                                | OFlags::CLOEXEC,
                            Mode::from_bits_truncate(0o600),
                        )
                        .map_err(|cause| error(path, cause.to_string()))?;
                        let mut file = File::from(fd);
                        file.write_all(input.as_bytes())
                            .and_then(|_| file.sync_all())
                            .map_err(|cause| error(path, cause.to_string()))?;
                    } else {
                        let next = if prefix.is_empty() {
                            component.into()
                        } else {
                            format!("{prefix}/{component}")
                        };
                        if !directories.contains_key(&next) {
                            mkdirat(directory, component, Mode::from_bits_truncate(0o700))
                                .map_err(|cause| error(path, cause.to_string()))?;
                            let opened = File::from(
                                openat(
                                    directory,
                                    component,
                                    OFlags::RDONLY
                                        | OFlags::DIRECTORY
                                        | OFlags::NOFOLLOW
                                        | OFlags::CLOEXEC,
                                    Mode::empty(),
                                )
                                .map_err(|cause| error(path, cause.to_string()))?,
                            );
                            directories.insert(next.clone(), opened);
                        }
                        prefix = next;
                    }
                }
            }
            for directory in directories.values() {
                directory
                    .sync_all()
                    .map_err(|cause| error(output_text, cause.to_string()))?;
            }
            let reopened = File::from(
                openat(
                    &parent_root.0,
                    leaf,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(|cause| error(output_text, cause.to_string()))?,
            );
            if Identity::of(
                &reopened
                    .metadata()
                    .map_err(|cause| error(output_text, cause.to_string()))?,
            )
            .file_id()
                != output_identity
            {
                return Err(error(
                    output_text,
                    "output directory was replaced during packaging",
                ));
            }
            parent_root
                .0
                .sync_all()
                .map_err(|cause| error(output_text, cause.to_string()))?;
            Ok(output)
        })();
        result.map_err(|cause: LiveRonError| {
            error(
                output_text,
                format!(
            "new output may be incomplete; inspect it before retrying with a new path: {cause}"
        ),
            )
        })
    }
}

#[cfg(not(unix))]
mod platform {
    use super::{error, LiveRonError};
    use std::path::{Path, PathBuf};
    pub(super) fn write_new_pack(
        _: &Path,
        _: &Path,
        _: &[(String, String)],
    ) -> Result<PathBuf, LiveRonError> {
        Err(error(
            "output",
            "no verified directory-relative no-follow filesystem backend on this platform",
        ))
    }
    #[derive(PartialEq, Eq)]
    pub(super) struct Identity;
    impl Identity {
        pub(super) fn file_id(&self) -> (u64, u64) {
            (0, 0)
        }
    }
    pub(super) struct Root;
    impl Root {
        pub(super) fn open(_: &Path) -> Result<Self, LiveRonError> {
            Err(error(
                "trusted_root",
                "no verified directory-relative no-follow filesystem backend on this platform",
            ))
        }
        pub(super) fn read(
            &self,
            path: &str,
            _: usize,
        ) -> Result<(String, Identity), LiveRonError> {
            Err(error(path, "no verified no-follow filesystem backend"))
        }
    }
}

#[cfg(test)]
mod tests;
