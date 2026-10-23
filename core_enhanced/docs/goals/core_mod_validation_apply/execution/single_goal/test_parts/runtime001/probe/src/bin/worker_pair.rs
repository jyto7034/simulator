//! Actual release binary identity admission. No Lab or approval is run.
use game_core::mod_validation::WorkerPair;
use serde::Serialize;
use serde_json::json;
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path};
fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=3{return Err("usage: worker_pair PLAIN_RELEASE_WORKER INSTRUMENTED_RELEASE_WORKER NEW_OUTPUT".into());}
    let plain=Path::new(&args[0]);let instrumented=Path::new(&args[1]);let out=Path::new(&args[2]);fs::create_dir(out)?;
    let environment="u11-identity-admission-only-no-performance-claim";
    let pair=WorkerPair::open(plain.to_owned(),instrumented.to_owned(),environment.into())?;
    u11_independent_public_probe::worker_evidence::preserve(&pair,&out.join("worker_identity"))?;
    save(&out.join("actual_pair.json"),pair.identity())?;
    let identity=pair.identity();
    assert_eq!(identity.plain.source_compiler_sha256,identity.instrumented.source_compiler_sha256);
    assert_eq!(identity.plain.target,identity.instrumented.target);
    assert_eq!(identity.plain.profile,"release");assert_eq!(identity.instrumented.profile,"release");
    assert_eq!(identity.plain.features,"");assert_eq!(identity.instrumented.features,"RUNTIME_OBSERVATION");
    assert_ne!(identity.plain.executable_sha256,identity.instrumented.executable_sha256);
    for (label,a,b,env) in [
        ("two_plain",plain,plain,environment.to_owned()),
        ("two_instrumented",instrumented,instrumented,environment.to_owned()),
        ("reversed_modes",instrumented,plain,environment.to_owned()),
        ("missing_environment",plain,instrumented,String::new()),
        ("oversized_environment",plain,instrumented,"x".repeat(1025)),
    ] {
        let error=WorkerPair::open(a.to_owned(),b.to_owned(),env).err().expect("invalid actual pair must fail admission");
        save(&out.join(format!("{label}.json")),&error)?;
    }
    let missing=out.join("intentionally_absent_worker");assert!(!missing.exists());
    for (label,a,b) in [("missing_plain",missing.as_path(),instrumented),("missing_instrumented",plain,missing.as_path())] {
        let error=WorkerPair::open(a.to_owned(),b.to_owned(),environment.into()).err().expect("absent worker cannot be paired");
        save(&out.join(format!("{label}.json")),&error)?;
    }
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"actual release pair identity admission",
        "actual_build_identity_processes_occurred":true,"negative_cases":7,
        "lab_executions":0,"approval_claimed":false,"performance_measurement_claimed":false}))?;
    println!("{}",json!({"passed":true,"output":out}));Ok(())
}
