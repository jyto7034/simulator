//! Default-release external consumer. Invoke each command in a separate process
//! against the SAME original for start → denied foreign host → restored Continue.
use game_server as host_server;
#[path="../../../shared/normal_save_support.rs"]
mod support;
use support::*;
use game_server::mod_admission::ApprovedPersistence;
use serde_json::json;
use std::{fs,path::PathBuf,sync::Arc};

fn main()->Result<()> {
    let admin_args:Vec<String>=std::env::args().skip(1).collect();
    if admin_args.first().is_some_and(|s|s=="--u11-accept") {
        if admin_args.len()!=2{return Err("--u11-accept requires one actual admission config".into());}
        let session=open(&config(std::path::Path::new(&admin_args[1]))?)?;
        let result=json!({"report":session.report,"host":session.query()?});
        if let Some(out)=std::env::var_os("U11_NORMAL_ADMIN_OUTPUT"){save(&PathBuf::from(out),&result)?;}
        println!("{}",serde_json::to_string(&result)?);return Ok(());
    }
    if let Some(result)=game_server::mod_admission::administration(&admin_args)? {
        if let Some(out)=std::env::var_os("U11_NORMAL_ADMIN_OUTPUT"){save(&PathBuf::from(out),&result)?;}
        println!("{}",serde_json::to_string(&result)?);return Ok(());
    }
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=5{return Err("usage: normal_save <init|reject|continue> <approved-config.json> <active-root> <profile-root> <new-evidence-dir>".into());}
    let command=args[0].to_str().ok_or("command must be UTF-8")?;
    let cfg=config(&PathBuf::from(&args[1]))?;
    let root=PathBuf::from(&args[2]);let profile=PathBuf::from(&args[3]);let out=PathBuf::from(&args[4]);
    fs::create_dir(&out)?;
    let session=open(&cfg)?;
    save(&out.join("actual_admission.json"),&session.report)?;
    let active_before=state(&root)?;let profile_before=state(&profile)?;
    save(&out.join("active_before.json"),&active_before.report())?;
    save(&out.join("profile_before.json"),&profile_before.report())?;
    match command {
        "init"=>{
            if root.exists()||profile.exists(){return Err("init requires absent disposable save roots".into());}
            let persistence=ApprovedPersistence::open_roots(root.clone(),profile.clone(),Arc::clone(&session.host))?;
            let(_,required,revision)=create_run(&session,&persistence,&out.join("start"))?;
            let stored=persistence.snapshot()?;
            assert!(stored.checkpoint.is_some());assert_eq!(stored.required_configuration.as_ref(),Some(&required));
            assert_eq!(stored.revision,revision.ok_or("real publication revision required")?);
            assert_eq!(active(&root.join("active_run.json"))?.unsigned.version,6);
            save(&out.join("RESULT.json"),&json!({"passed":true,"command":command,"revision":revision,"required":required}))?;
        }
        "reject"=>{
            let error=protected_error(ApprovedPersistence::open_roots(root.clone(),profile.clone(),Arc::clone(&session.host)))?;
            active_before.ensure_unchanged(&state(&root)?)?;profile_before.ensure_unchanged(&state(&profile)?)?;
            save(&out.join("RESULT.json"),&json!({"passed":true,"command":command,"error":error,"exact_originals_unchanged":true}))?;
        }
        "continue"=>{
            let persistence=ApprovedPersistence::open_roots(root.clone(),profile.clone(),Arc::clone(&session.host))?;
            continue_original(&session,&persistence,&out.join("continue.json"))?;
            drop(persistence);
            active_before.ensure_unchanged(&state(&root)?)?;profile_before.ensure_unchanged(&state(&profile)?)?;
            save(&out.join("RESULT.json"),&json!({"passed":true,"command":command,"same_original_continued":true,"exact_originals_unchanged":true}))?;
        }
        _=>return Err("unknown normal save command".into()),
    }
    save(&out.join("active_after.json"),&state(&root)?.report())?;
    save(&out.join("profile_after.json"),&state(&profile)?.report())?;
    Ok(())
}
