//! Copy a test-owned issuer cache into a fresh private directory for fault tests.
//! Worker evidence stays at its original source; this copy starts with no runs.
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path};
pub fn private_directory(path:&Path)->Result<(),Box<dyn Error>> {
    #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;
        let mut b=fs::DirBuilder::new();b.mode(0o700);b.create(path)?;Ok(())}
    #[cfg(not(unix))] {let _=path;Err("private Unix environment required".into())}
}
pub fn copy(source:&Path,destination:&Path)->Result<(),Box<dyn Error>> {
    private_directory(destination)?;
    let mut count=0usize;let mut total=0u64;
    for entry in fs::read_dir(source)? {let entry=entry?;let ty=entry.file_type()?;
        if ty.is_dir()&&entry.file_name()=="workers" {continue;}
        count+=1;
        if !ty.is_file()||count>256||entry.metadata()?.len()>8*1024*1024 {return Err("unexpected or oversized private fixture entry".into());}
        let bytes=fs::read(entry.path())?;total=total.checked_add(bytes.len() as u64).ok_or("private fixture byte overflow")?;
        if total>256*1024*1024{return Err("private fixture copy total bound exceeded".into());}
        let mut options=OpenOptions::new();options.write(true).create_new(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
        options.open(destination.join(entry.file_name()))?.write_all(&bytes)?;
    }
    private_directory(&destination.join("workers"))?;Ok(())
}
