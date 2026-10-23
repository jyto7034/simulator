//! Exact original-file oracle for cache/preflight tests. Reports contain hashes,
//! never secret/key or save bytes. Capture outside timed allocator intervals.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::{self,Read}, path::{Path, PathBuf}, time::UNIX_EPOCH};

#[derive(PartialEq, Eq)]
struct Entry {
    kind: &'static str,
    bytes: Vec<u8>,
    link: Option<PathBuf>,
    modified_ns: Option<u128>,
    mode: Option<u32>,
}
#[derive(PartialEq, Eq)]
pub struct FileState {
    exists: bool,
    entries: BTreeMap<PathBuf, Entry>,
}
#[derive(Serialize)]
pub struct FileStateReport {
    pub root_exists: bool,
    pub entries: Vec<EntryReport>,
}
#[derive(Serialize)]
pub struct EntryReport {
    pub relative: PathBuf,
    pub kind: &'static str,
    pub bytes: usize,
    pub sha256: Option<String>,
    pub symlink_target: Option<PathBuf>,
    pub modified_ns: Option<u128>,
    pub mode: Option<u32>,
}
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
impl FileState {
    /// Caller supplies finite probe bounds. Exceeding them is a probe failure,
    /// never a product limit/criterion or a successful unchanged comparison.
    pub fn capture(root: &Path, max_entries: usize, max_total_bytes: usize) -> io::Result<Self> {
        let mut state = Self { exists: false, entries: BTreeMap::new() };
        match fs::symlink_metadata(root) {
            Err(error) if error.kind()==io::ErrorKind::NotFound => return Ok(state),
            Err(error) => return Err(error),
            Ok(_) => state.exists = true,
        }
        let mut used = 0usize;
        state.visit(root, root, max_entries, max_total_bytes, &mut used)?;
        Ok(state)
    }
    fn visit(&mut self, root: &Path, current: &Path, max_entries: usize,
             max_total_bytes: usize, used: &mut usize) -> io::Result<()> {
        if self.entries.len()>=max_entries { return Err(invalid("file oracle entry bound exceeded")); }
        let metadata = fs::symlink_metadata(current)?;
        let modified_ns = metadata.modified().ok().and_then(|v|v.duration_since(UNIX_EPOCH).ok()).map(|v|v.as_nanos());
        #[cfg(unix)]
        let mode = { use std::os::unix::fs::PermissionsExt; Some(metadata.permissions().mode()) };
        #[cfg(not(unix))]
        let mode = None;
        let mut entry = Entry { kind:"directory",bytes:vec![],link:None,modified_ns,mode };
        if metadata.file_type().is_symlink() {
            entry.kind="symlink"; entry.link=Some(fs::read_link(current)?);
        } else if metadata.is_file() {
            entry.kind="file";
            let length = usize::try_from(metadata.len()).map_err(|_|invalid("file length overflow"))?;
            let next = used.checked_add(length).ok_or_else(||invalid("file oracle byte sum overflow"))?;
            if next>max_total_bytes { return Err(invalid("file oracle byte bound exceeded")); }
            let remaining=max_total_bytes.checked_sub(*used).ok_or_else(||invalid("file oracle byte bound exceeded"))?;
            let cap=u64::try_from(remaining).map_err(|_|invalid("file oracle read bound overflow"))?
                .checked_add(1).ok_or_else(||invalid("file oracle read bound overflow"))?;
            fs::File::open(current)?.take(cap).read_to_end(&mut entry.bytes)?;
            let after=fs::symlink_metadata(current)?;
            if entry.bytes.len()!=length || !after.is_file() || after.file_type().is_symlink()
                || after.len()!=metadata.len() || after.modified().ok()!=metadata.modified().ok() {
                return Err(invalid("file changed during original snapshot"));
            }
            *used=next;
        } else if !metadata.is_dir() { return Err(invalid("special file in original snapshot")); }
        let relative=current.strip_prefix(root).map_err(|_|invalid("snapshot escaped root"))?.to_owned();
        self.entries.insert(relative,entry);
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            let mut children=fs::read_dir(current)?.collect::<Result<Vec<_>,_>>()?;
            children.sort_by_key(|entry|entry.file_name());
            for child in children { self.visit(root,&child.path(),max_entries,max_total_bytes,used)?; }
        }
        Ok(())
    }
    pub fn report(&self)->FileStateReport {
        FileStateReport { root_exists:self.exists, entries:self.entries.iter().map(|(relative,entry)|EntryReport {
            relative:relative.clone(),kind:entry.kind,bytes:entry.bytes.len(),
            sha256:(entry.kind=="file").then(||format!("{:x}",Sha256::digest(&entry.bytes))),
            symlink_target:entry.link.clone(),modified_ns:entry.modified_ns,mode:entry.mode,
        }).collect() }
    }
    /// Full in-memory byte comparison, plus membership/type/mtime/mode. Errors
    /// name only the path so keys and private save bytes cannot enter a log.
    pub fn ensure_unchanged(&self, after:&Self)->io::Result<()> {
        if self.exists!=after.exists { return Err(invalid("original root existence changed")); }
        if self.entries.keys().ne(after.entries.keys()) { return Err(invalid("original directory membership changed")); }
        for (path,before) in &self.entries {
            if after.entries.get(path)!=Some(before) { return Err(invalid(format!("original entry changed: {}",path.display()))); }
        }
        Ok(())
    }
    /// Legitimate publication may add entries and update parent directory mtimes.
    /// Every existing regular file/link must retain exact content and metadata.
    pub fn ensure_original_files_unchanged(&self, after:&Self)->io::Result<()> {
        if self.exists&&!after.exists{return Err(invalid("original root disappeared"));}
        for(path,before)in &self.entries {
            let next=after.entries.get(path).ok_or_else(||invalid(format!("original entry disappeared: {}",path.display())))?;
            if before.kind=="directory" {
                if next.kind!=before.kind||next.mode!=before.mode{return Err(invalid(format!("original directory changed type/mode: {}",path.display())));}
            }else if next!=before{return Err(invalid(format!("original file changed: {}",path.display())));}
        }
        Ok(())
    }
}
