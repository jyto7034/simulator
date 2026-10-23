//! Preserve exact actual worker identity process evidence before consuming pair.
use game_core::mod_validation::WorkerPair;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs::{self,OpenOptions},io::{Read,Write},path::Path};
pub fn preserve(pair:&WorkerPair,out:&Path)->Result<(),Box<dyn Error>> {
    fs::create_dir(out)?;assert_eq!(pair.evidence_paths().len(),2);
    let mut records=Vec::new();
    for (index,path) in pair.evidence_paths().iter().enumerate() {
        for (kind,source) in [("stdout",path.clone()),("stderr",path.with_extension("stderr")),
            ("process",path.with_extension("process.json"))] {
            let mut bytes=Vec::new();fs::File::open(&source)?.take(65_537).read_to_end(&mut bytes)?;
            assert!(bytes.len()<=65_536,"bounded identity evidence file");
            if kind=="process" {
                let process:Value=serde_json::from_slice(&bytes)?;
                assert_eq!(process["child_reaped"],json!(true));assert_eq!(process["success"],json!(true));
                let expected=if index==0 {&pair.identity().plain}else{&pair.identity().instrumented};
                assert_eq!(process["binary_sha256_before"],json!(expected.executable_sha256));
                assert_eq!(process["binary_sha256_after"],json!(expected.executable_sha256));
            }
            let destination=out.join(format!("{index}-{kind}.raw"));
            OpenOptions::new().write(true).create_new(true).open(&destination)?.write_all(&bytes)?;
            records.push(json!({"source":source,"preserved":destination,"bytes":bytes.len(),
                "sha256":format!("{:x}",Sha256::digest(&bytes))}));
        }
    }
    let bytes=serde_json::to_vec_pretty(&json!({"actual_identity_processes":true,"records":records}))?;
    OpenOptions::new().write(true).create_new(true).open(out.join("manifest.json"))?.write_all(&bytes)?;Ok(())
}
