//! Read the probe's own issuer key only in memory, and check explicitly named
//! public artifacts for accidental raw/JSON/hex/base64 disclosure. Never print a
//! key, pattern, artifact body, or a Debug representation of this private state.
use serde::Serialize;
use std::{fs,io::{self,Read},path::{Path,PathBuf}};

#[derive(Serialize)]
pub struct SecretAbsenceReport {pub checked_files:usize,pub checked_bytes:usize,pub key_material_exported:bool}
fn invalid(message:impl Into<String>)->io::Error {io::Error::new(io::ErrorKind::InvalidData,message.into())}
fn read_bounded(path:&Path,cap:usize)->io::Result<Vec<u8>> {
    let metadata=fs::symlink_metadata(path)?;
    if !metadata.is_file()||metadata.file_type().is_symlink(){return Err(invalid("secret oracle requires an explicit regular file"));}
    let cap_u64=u64::try_from(cap).map_err(|_|invalid("secret oracle byte bound overflow"))?;
    if metadata.len()>cap_u64{return Err(invalid("secret oracle byte bound exceeded"));}
    let read_cap=cap_u64.checked_add(1).ok_or_else(||invalid("secret oracle byte bound overflow"))?;
    let mut bytes=Vec::new();fs::File::open(path)?.take(read_cap).read_to_end(&mut bytes)?;
    if bytes.len()>cap||bytes.len() as u64!=metadata.len(){return Err(invalid("file changed or exceeded secret oracle bound"));}
    Ok(bytes)
}
fn base64(bytes:&[u8],url_safe:bool)->Vec<u8> {
    // Encoding solely for leak detection. This is not a MAC, authentication
    // primitive, key derivation, or product cache implementation.
    let table=if url_safe {b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"}
        else {b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"};
    let mut out=Vec::new();
    for chunk in bytes.chunks(3) {
        let word=(chunk[0] as u32)<<16 | (chunk.get(1).copied().unwrap_or(0) as u32)<<8
            | chunk.get(2).copied().unwrap_or(0) as u32;
        out.push(table[((word>>18)&63) as usize]);out.push(table[((word>>12)&63) as usize]);
        out.push(if chunk.len()>1 {table[((word>>6)&63) as usize]}else{b'='});
        out.push(if chunk.len()>2 {table[(word&63) as usize]}else{b'='});
    }
    out
}
pub fn require_secret_absent(private_root:&Path,public_artifacts:&[PathBuf],max_total_bytes:usize)
    ->io::Result<SecretAbsenceReport> {
    let key=read_bounded(&private_root.join("issuer.key"),48)?;
    if key.len()!=48{return Err(invalid("unexpected actual issuer key codec; update independent privacy oracle explicitly"));}
    let secret=&key[16..];let mut patterns=vec![key.clone(),secret.to_vec()];
    for bytes in [&key[..],secret] {
        let hex=bytes.iter().map(|byte|format!("{byte:02x}")).collect::<String>();
        patterns.push(hex.as_bytes().to_vec());patterns.push(hex.to_ascii_uppercase().into_bytes());
        patterns.push(serde_json::to_vec(bytes).map_err(|_|invalid("private pattern encoding failed"))?);
        let spaced=format!("[{}]",bytes.iter().map(u8::to_string).collect::<Vec<_>>().join(", "));
        patterns.push(spaced.into_bytes());
        for url_safe in [false,true] {
            let mut encoded=base64(bytes,url_safe);patterns.push(encoded.clone());
            while encoded.last()==Some(&b'='){encoded.pop();}
            patterns.push(encoded);
        }
    }
    let mut checked_bytes=0usize;
    for path in public_artifacts {
        let remaining=max_total_bytes.checked_sub(checked_bytes).ok_or_else(||invalid("secret oracle total byte bound exceeded"))?;
        let raw=read_bounded(path,remaining)?;
        for pattern in &patterns {
            if raw.windows(pattern.len()).any(|window|window==pattern.as_slice()) {
                return Err(invalid(format!("issuer key material occurred in public artifact {}",path.display())));
            }
        }
        checked_bytes=checked_bytes.checked_add(raw.len()).ok_or_else(||invalid("secret oracle byte count overflow"))?;
    }
    Ok(SecretAbsenceReport{checked_files:public_artifacts.len(),checked_bytes,key_material_exported:false})
}
