//! Independent integration tests over actual approved host and persistence.
//! Requires Root-reviewed calibration/config files supplied by the sole runner.
//! Run explicit parent selectors with --test-threads=1. The ignored child is a
//! subprocess helper, never counted as an independent successful test.
use crate as host_server;
#[path="normal_save_support.rs"]
mod support;
#[path="normal_actor_tests.rs"]
mod actors;
#[path="normal_update_tests.rs"]
mod updates;
use support::*;
use crate::mod_admission::ApprovedPersistence;
use game_core::mod_validation::JobState;
use serde_json::{json,Value};
use std::{fs,path::{Path,PathBuf},process::Command,sync::Arc};

fn inputs()->Result<(Cases,PathBuf)>{
    let file=std::env::var_os("U11_NORMAL_SAVE_CASES").ok_or("actual reviewed normal save case manifest required")?;
    let cases=serde_json::from_slice(&fs::read(file)?)?;
    let out=std::env::var_os("U11_NORMAL_SAVE_OUTPUT").ok_or("exclusive normal save output parent required")?;
    Ok((cases,PathBuf::from(out)))
}
fn output(name:&str)->Result<(Cases,PathBuf)>{let(cases,parent)=inputs()?;let out=parent.join(name);fs::create_dir(&out)?;Ok((cases,out))}
fn disposable_configs(cases:&Cases,out:&Path)->Result<Cases>{
    let mut a=config(&cases.a)?;let mut b=config(&cases.b)?;let mut c=config(&cases.c)?;
    assert_eq!(a.issuer_root,b.issuer_root,"shared normal host issuer required");
    assert_eq!(a.issuer_root,c.issuer_root,"same issuer is required for meaningful staged selection");
    let original_cache=state(&a.issuer_root)?;
    let copied=out.join("private_issuer");cache_fixture::copy(&a.issuer_root,&copied)?;
    original_cache.ensure_unchanged(&state(&a.issuer_root)?)?;
    a.issuer_root=copied.clone();b.issuer_root=copied.clone();c.issuer_root=copied;
    let paths=Cases{a:out.join("a.json"),b:out.join("b.json"),c:out.join("c.json")};
    save(&paths.a,&a)?;save(&paths.b,&b)?;save(&paths.c,&c)?;Ok(paths)
}
fn closed_run(config_path:&Path,root:&Path,out:&Path)->Result<()> {
    let session=open(&config(config_path)?)?;
    let persistence=ApprovedPersistence::open(root,Arc::clone(&session.host))?;
    let (_,required,_)=create_run(&session,&persistence,out)?;
    let snapshot=persistence.snapshot()?;
    assert!(snapshot.checkpoint.is_some());assert_eq!(snapshot.required_configuration.as_ref(),Some(&required));
    let wire=active(&root.join("active_run.json"))?;assert_eq!(wire.unsigned.version,6);
    assert_eq!(wire.unsigned.required_configuration.as_ref(),Some(&required));
    let core:Value=serde_json::from_slice(&snapshot.checkpoint.unwrap().core_payload_json)?;assert_eq!(core["version"],11);
    Ok(())
}
fn denied(root:&Path,profile:&Path,config_path:&Path,out:&Path)->Result<()> {
    let session=open(&config(config_path)?)?;
    let active_before=state(root)?;let profile_before=state(profile)?;
    let error=protected_error(ApprovedPersistence::open_roots(root.to_owned(),profile.to_owned(),Arc::clone(&session.host)))?;
    active_before.ensure_unchanged(&state(root)?)?;profile_before.ensure_unchanged(&state(profile)?)?;
    save(out,&json!({"error":error,"active_before":active_before.report(),"active_after":state(root)?.report(),
        "profile_before":profile_before.report(),"profile_after":state(profile)?.report(),"original_unchanged":true}))?;
    Ok(())
}

#[test]
#[ignore="explicit management entry; actual diagnostic/adoption is not a functional pass"]
fn u11_normal_host_calibration_administration(){
    let run=||->Result<()>{
        let args:Vec<String>=serde_json::from_slice(&fs::read(std::env::var_os("U11_NORMAL_ADMIN_ARGS")
            .ok_or("actual administration argument file required")?)?)?;
        let out=PathBuf::from(std::env::var_os("U11_NORMAL_ADMIN_OUTPUT").ok_or("new actual administration result path required")?);
        let result=if args.first().is_some_and(|s|s=="--u11-accept") {
            if args.len()!=2{return Err("--u11-accept requires one actual admission config".into());}
            let session=open(&config(Path::new(&args[1]))?)?;
            json!({"report":session.report,"host":session.query()?})
        }else{crate::mod_admission::administration(&args)?.ok_or("calibration administration command required")?};
        save(&out,&result)?;Ok(())
    };run().unwrap();
}

#[test]
#[ignore="requires actual reviewed same-executable calibration cases and exclusive evidence; run explicit U11 selector"]
fn u11_normal_missing_mod_preflight_preserves_same_original_and_restored_continue(){
    let run=||->Result<()>{
        let(cases,out)=output("same_original")?;let cases=disposable_configs(&cases,&out)?;
        let original=out.join("original");closed_run(&cases.a,&original,&out.join("start_a"))?;
        let snapshot=state(&original)?;save(&out.join("original_inventory.json"),&snapshot.report())?;
        // Existing original, including profile and both backups, stays byte-for-
        // byte unchanged while approved B lacks the save's required A selection.
        denied(&original,&original,&cases.b,&out.join("missing_a_denied.json"))?;
        snapshot.ensure_unchanged(&state(&original)?)?;
        // A second profile root is intentionally absent. Rejection must precede
        // mkdir, lock creation, profile bootstrap and backup repair.
        let separate=out.join("separate");copy_tree(&original,&separate)?;no_locks(&separate)?;
        let absent_profile=out.join("profile_must_remain_absent");
        denied(&separate,&absent_profile,&cases.b,&out.join("no_directory_or_lock.json"))?;
        assert!(!absent_profile.exists());assert!(!separate.join("active_run.lock").exists());
        let session=open(&config(&cases.a)?)?;
        let persistence=ApprovedPersistence::open(&original,Arc::clone(&session.host))?;
        continue_original(&session,&persistence,&out.join("same_original_continue.json"))?;
        snapshot.ensure_unchanged(&state(&original)?)?;
        drop(persistence);drop(session);
        // Real compatible backup recovery remains available. Missing A must not
        // repair the corrupted current before the approved configuration returns.
        let damaged=out.join("corrupt_current");copy_tree(&original,&damaged)?;no_locks(&damaged)?;
        let backup=fs::read(damaged.join("active_run.backup.json"))?;
        fs::write(damaged.join("active_run.json"),b"{truncated-current")?;
        denied(&damaged,&damaged,&cases.b,&out.join("missing_a_cannot_repair.json"))?;
        let session=open(&config(&cases.a)?)?;
        let persistence=ApprovedPersistence::open(&damaged,Arc::clone(&session.host))?;
        continue_original(&session,&persistence,&out.join("confirmed_backup_continue.json"))?;
        assert_eq!(fs::read(damaged.join("active_run.json"))?,backup,"repair preserves exact compatible backup wire");
        drop(persistence);drop(session);
        // Healthy legacy V5 is a read-only exact identity case; opening and
        // continuing may not silently rewrite it as V6.
        let legacy=out.join("legacy_v5");copy_tree(&original,&legacy)?;
        for name in ["active_run.json","active_run.backup.json"] {
            let path=legacy.join(name);let mut wire=active(&path)?;
            wire.unsigned.version=5;wire.unsigned.required_configuration=None;write_active(&path,wire)?;
        }
        let legacy_before=state(&legacy)?;
        denied(&legacy,&legacy,&cases.b,&out.join("legacy_foreign_identity_denied.json"))?;
        let session=open(&config(&cases.a)?)?;
        let persistence=ApprovedPersistence::open(&legacy,Arc::clone(&session.host))?;
        continue_original(&session,&persistence,&out.join("legacy_exact_original_continue.json"))?;
        legacy_before.ensure_unchanged(&state(&legacy)?)?;
        save(&out.join("RESULT.json"),&json!({"passed":true,"normal_approved_path":true,"same_original_continued":true,
            "missing_mod_writer_effects":0,"legacy_version_5_wire_preserved":true}))?;Ok(())
    };run().unwrap();
}

// A saved active run does not make StartNewGame legal on a running Core. Like
// the normal fresh actor/title path, keep the durable revision and prepare a
// new NotStarted consumer under the actual currently approved generation.
fn dormant_application(session:&host_server::mod_admission::AdmissionSession,persistence:&ApprovedPersistence)
    ->Result<(game_core::application::CoreApplication,game_core::mod_validation::RequiredConfiguration)>{
    let (mut app,required)=session.host.new_application(77)?;
    app.merge_player_profile(serde_json::to_value(persistence.player_profile()?)?.into())?;
    assert!(app.is_not_started());
    assert!(run_snapshot(&app)?["action_capabilities"].as_array().unwrap().iter()
        .any(|cap|cap["action"]=="StartNewGame"&&cap["enabled"]==true));
    Ok((app,required))
}
fn inventory(app:&game_core::application::CoreApplication)->Result<Value>{
    Ok(app.snapshot(game_core::application::ApplicationSnapshotKind::Inventory)?.read_model().as_value().clone())
}

#[test]
#[ignore="requires actual reviewed same-executable calibration cases and exclusive evidence; run explicit U11 selector"]
fn u11_durable_b_remains_usable_when_future_c_is_installed(){
    let run=||->Result<()>{
        let(cases,out)=output("durable_b_future_c")?;let cases=disposable_configs(&cases,&out)?;
        let mut session=open(&config(&cases.a)?)?;
        let root=out.join("save");let persistence=ApprovedPersistence::open(&root,Arc::clone(&session.host))?;
        let(active_a,a,mut revision)=create_run(&session,&persistence,&out.join("start_a"))?;
        let active_a_before=run_snapshot(&active_a)?;let durable_a_before=state(&root)?;
        let(mut app,mut required)=dormant_application(&session,&persistence)?;
        assert_eq!(required,a);durable_a_before.ensure_unchanged(&state(&root)?)?;
        let accepted_b=session.validate_and_apply(acquire(&config(&cases.b)?)?)?;
        assert_eq!(accepted_b.state,JobState::Passed);save(&out.join("b_accepted.json"),&accepted_b)?;
        execute(&session,&persistence,&mut app,&mut required,&mut revision,
            json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":true}),true)?;
        finish_start(&session,&persistence,&mut app,&mut required,&mut revision,&out.join("start_b"))?;
        let b=required.clone();assert_ne!(a,b);
        assert_eq!(run_snapshot(&active_a)?,active_a_before);
        assert_eq!(active_a.content_identity(),&a.content_identity);
        let(mut c_shell,c_shell_required)=dormant_application(&session,&persistence)?;
        let(mut b_reader,b_reader_required)=dormant_application(&session,&persistence)?;
        assert_eq!(c_shell_required,b);assert_eq!(b_reader_required,b);
        let stored_b=persistence.snapshot()?;assert!(stored_b.checkpoint.is_some());
        assert_eq!(stored_b.required_configuration.as_ref(),Some(&b));
        let accepted_c=session.validate_and_apply(acquire(&config(&cases.c)?)?)?;
        assert_eq!(accepted_c.state,JobState::Passed);save(&out.join("c_accepted.json"),&accepted_c)?;
        let c=session.query()?.required;assert_ne!(a,c);assert_ne!(b,c);
        assert_eq!(app.content_identity(),&b.content_identity);
        assert_eq!(persistence.snapshot()?.required_configuration.as_ref(),Some(&b),
            "durable B does not lose admission when future C replaces the future slot");
        // Use a real eligible B equipment edit for autosave. In-run checkpoint
        // load is unavailable before a SavePoint; its contract remains covered
        // by existing U04 regressions and the same-original Continue test.
        let inventory_before=inventory(&app)?;
        let selected=inventory_before["equipments"].as_array().unwrap().iter()
            .find(|item|item["item"]["definition_id"]=="standard_armor"
                &&item["item"]["can_unequip"]==true&&item["equipped_to"].is_string())
            .ok_or("actual removable owned armor required")?;
        let item=selected["instance_uuid"].clone();let owner=selected["equipped_to"].clone();
        let result=execute(&session,&persistence,&mut app,&mut required,&mut revision,
            json!({"type":"un_equip_item","item_uuid":item,"target_unit":owner}),false)?;
        let inventory_after=inventory(&app)?;
        let edited=inventory_after["equipments"].as_array().unwrap().iter()
            .find(|entry|entry["instance_uuid"]==item).ok_or("same owned armor must survive unequip")?;
        assert!(edited["equipped_to"].is_null());assert_ne!(inventory_before,inventory_after);
        let after=persistence.snapshot()?;
        assert_eq!(required,b);assert_eq!(after.required_configuration.as_ref(),Some(&b));
        assert_ne!(stored_b.revision,after.revision,"B autosave must actually advance durable revision");
        let old_payload=&stored_b.checkpoint.as_ref().unwrap().core_payload_json;
        let new_checkpoint=after.checkpoint.as_ref().ok_or("edited B needs a nonempty durable continuation")?;
        assert_ne!(old_payload,&new_checkpoint.core_payload_json,"no-op forced publication cannot satisfy this oracle");
        b_reader.continue_from_active_run_checkpoint(PLAYER,&new_checkpoint.to_application())?;
        assert_eq!(b_reader.content_identity(),&b.content_identity);
        assert_eq!(inventory(&b_reader)?,inventory_after,"durable B must restore the same exact equipment-copy owner change");
        save(&out.join("b_equipment_autosave_after_c.json"),&json!({"command_result":result,"item_uuid":item,
            "old_owner":owner,"new_owner":Value::Null,"before":inventory_before,"after":inventory_after,
            "old_payload_sha256":digest(old_payload),"new_payload_sha256":digest(&new_checkpoint.core_payload_json),
            "old_revision":stored_b.revision,"new_revision":after.revision,"required":required,
            "same_durable_b_roundtrip_inventory":inventory(&b_reader)?}))?;
        assert_eq!(app.content_identity(),&b.content_identity);
        assert_eq!(run_snapshot(&active_a)?,active_a_before);
        let before_replacement=run_snapshot(&app)?;let shell_before=run_snapshot(&c_shell)?;
        let durable_before_drop=state(&root)?;
        let command=game_core::application::ApplicationCommand::from_document(json!({"type":"start_new_game",
            "game_mode":"Standard","replace_active_run":true}).into()).map_err(|e|format!("replace command: {e:?}"))?;
        let (draft,next)=session.host.prepare_with_future_generation(&mut c_shell,&c_shell_required,PLAYER,command,Default::default())
            .map_err(|e|format!("C staged replacement: {e:?}"))?;
        assert_eq!(next,c);drop(draft);
        assert_eq!(run_snapshot(&c_shell)?,shell_before);assert_eq!(run_snapshot(&app)?,before_replacement);
        assert_eq!(app.content_identity(),&b.content_identity);assert_eq!(run_snapshot(&active_a)?,active_a_before);
        assert_eq!(persistence.snapshot()?.revision,after.revision);durable_before_drop.ensure_unchanged(&state(&root)?)?;
        save(&out.join("RESULT.json"),&json!({"passed":true,"a":a,"durable_b":b,"future_c":c,
            "b_actual_autosave_after_c":true,"uncommitted_c_replacement_preserved_b":true}))?;Ok(())
    };run().unwrap();
}

#[test]
#[ignore="subprocess-only crash helper; execute parent u11_accepted_v2 tests"]
fn u11_normal_save_crash_child(){
    let run=||->Result<()>{
        let cases:Cases=serde_json::from_slice(&fs::read(std::env::var_os("U11_CHILD_CASES").ok_or("child cases required")?)?)?;
        let out=PathBuf::from(std::env::var_os("U11_CHILD_OUTPUT").ok_or("child output required")?);
        let stage=std::env::var("U11_CHILD_CRASH_STAGE")?;
        let mut session=open(&config(&cases.a)?)?;
        let root=out.join("save");let persistence=ApprovedPersistence::open(&root,Arc::clone(&session.host))?;
        let(active_a,required_a,mut revision)=create_run(&session,&persistence,&out.join("start_a"))?;
        let active_a_before=run_snapshot(&active_a)?;let durable_a_before=state(&root)?;
        let(mut app,mut required)=dormant_application(&session,&persistence)?;
        assert_eq!(required,required_a);durable_a_before.ensure_unchanged(&state(&root)?)?;
        save(&out.join("accepted_before.json"),&json!({"revision":revision,"required":required,
            "inventory":state(&root)?.report()}))?;
        fs::copy(root.join("active_run.json"),out.join("before_active.json"))?;
        let target=if std::env::var("U11_CHILD_SAME_A").ok().as_deref()==Some("1"){&cases.a}else{&cases.b};
        let result=session.validate_and_apply(acquire(&config(target)?)?)?;assert_eq!(result.state,JobState::Passed);
        save(&out.join("validated_transition.json"),&result)?;
        assert_eq!(run_snapshot(&active_a)?,active_a_before);
        assert_eq!(active_a.content_identity(),&required_a.content_identity);
        durable_a_before.ensure_unchanged(&state(&root)?)?;
        // Only this eligible fresh-shell transition replaces the same original
        // durable A and reaches the existing actual U04 filesystem crash seam.
        std::env::set_var("U04_DURABLE_CRASH_AT",stage);
        execute(&session,&persistence,&mut app,&mut required,&mut revision,
            json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":true}),true)?;
        Err("actual durable crash stage was never reached".into())
    };run().unwrap();
}
fn crash(cases:&Cases,out:&Path,stage:&str,same_a:bool)->Result<PathBuf>{
    fs::create_dir(out)?;
    let paths=disposable_configs(cases,out)?;let manifest=out.join("cases.json");save(&manifest,&paths)?;
    let result=Command::new(std::env::current_exe()?).args(["--exact",
        "mod_admission::u11_normal_save_tests::u11_normal_save_crash_child","--ignored","--nocapture","--test-threads=1"])
        .env("U11_CHILD_CASES",manifest).env("U11_CHILD_OUTPUT",out).env("U11_CHILD_CRASH_STAGE",stage)
        .env("U11_CHILD_SAME_A",if same_a{"1"}else{"0"}).env_remove("U04_DURABLE_CRASH_AT").output()?;
    fs::write(out.join("child.stdout"),&result.stdout)?;fs::write(out.join("child.stderr"),&result.stderr)?;
    save(&out.join("child_exit.json"),&json!({"exit":result.status.code(),"actual_stage":stage}))?;
    assert_eq!(result.status.code(),Some(86),"must crash at actual completed filesystem stage");
    Ok(out.join("save"))
}
fn expected_new(root:&Path)->Result<Value>{
    let journal:Value=serde_json::from_slice(&fs::read(root.join("application_commit.pending.json"))?)?;
    assert_eq!(journal["intent"]["version"],2);
    assert_eq!(journal["intent"]["old_active"]["revision"],journal["intent"]["expected_revision"]);
    assert_ne!(journal["intent"]["active"]["revision"],journal["intent"]["expected_revision"]);
    assert!(journal["intent"]["old_required_configuration"].is_object());
    assert!(journal["intent"]["active"]["required_configuration"].is_object());
    Ok(journal["intent"]["active"].clone())
}
// Compatibility fixture only: preserve the historical field order and omit
// every V2 field. It derives its payload from an actual accepted transaction;
// it never creates an approval or supplies authority for a V2 transition.
fn legacy_v1(root:&Path)->Result<Value>{
    use serde::{Deserialize,Serialize};
    use crate::persistence::active_run_store::StoredCheckpoint;
    use game_core::game::encounter_compendium::PlayerProfileState;
    #[derive(Serialize,Deserialize)]
    struct Record {revision:uuid::Uuid,checkpoint:Option<StoredCheckpoint>}
    #[derive(Serialize)]
    struct Intent {version:u32,expected_revision:uuid::Uuid,active:Record,profile:PlayerProfileState}
    #[derive(Serialize)]
    struct Envelope {intent:Intent,checksum_sha256:String}
    let path=root.join("application_commit.pending.json");
    let original:Value=serde_json::from_slice(&fs::read(&path)?)?;
    let mut record=original["intent"]["active"].clone();
    record.as_object_mut().ok_or("accepted active record required")?.remove("required_configuration");
    let intent=Intent{version:1,expected_revision:serde_json::from_value(original["intent"]["expected_revision"].clone())?,
        active:serde_json::from_value(record.clone())?,profile:serde_json::from_value(original["intent"]["profile"].clone())?};
    let checksum_sha256=digest(&serde_json::to_vec(&intent)?);
    fs::write(path,serde_json::to_vec_pretty(&Envelope{intent,checksum_sha256})?)?;
    record["required_configuration"]=Value::Null;Ok(record)
}
fn recover_b(root:&Path,config_path:&Path,expected:&Value,out:&Path)->Result<()> {
    let session=open(&config(config_path)?)?;
    let store=ApprovedPersistence::open(root,Arc::clone(&session.host))?;let snapshot=store.snapshot()?;
    assert_eq!(serde_json::to_value(snapshot.revision)?,expected["revision"]);
    assert_eq!(serde_json::to_value(&snapshot.checkpoint)?,expected["checkpoint"]);
    assert_eq!(serde_json::to_value(&snapshot.required_configuration)?,expected["required_configuration"]);
    assert!(!root.join("application_commit.pending.json").exists());
    assert_eq!(fs::read(root.join("active_run.json"))?,fs::read(root.join("active_run.backup.json"))?);
    save(out,&json!({"recovered_revision":snapshot.revision,"required":snapshot.required_configuration,
        "profile":store.player_profile()?,"authoritative_intent_cleared":true}))?;Ok(())
}

#[test]
#[ignore="requires actual reviewed same-executable calibration cases and exclusive evidence; run explicit U11 selector"]
fn u11_accepted_v2_a_to_b_crash_boundaries_and_exact_original_protection(){
    let run=||->Result<()>{
        let(cases,out)=output("accepted_v2")?;
        // Preaccept temp has no authority to replace incompatible A under B.
        let pre=crash(&cases,&out.join("before_accept"),"intent_temp_synced",false)?;
        assert!(pre.join("application_commit.pending.tmp").is_file());assert!(!pre.join("application_commit.pending.json").exists());
        let paths:Cases=serde_json::from_slice(&fs::read(out.join("before_accept/cases.json"))?)?;
        no_locks(&pre)?;denied(&pre,&pre,&paths.b,&out.join("before_accept_b_denied.json"))?;
        let session=open(&config(&paths.a)?)?;let store=ApprovedPersistence::open(&pre,Arc::clone(&session.host))?;
        continue_original(&session,&store,&out.join("before_accept_same_a_continue.json"))?;drop(store);drop(session);
        let accepted=crash(&cases,&out.join("at_accept"),"intent_renamed",false)?;
        let expected=expected_new(&accepted)?;
        let paths:Cases=serde_json::from_slice(&fs::read(out.join("at_accept/cases.json"))?)?;
        let accepted_original=state(&accepted)?;
        // Originals are copied into disposable variants; source accepted A→B
        // transaction remains available unchanged for the final positive.
        for variant in ["missing_b","wrong_payload_same_revision","unsupported_current","unsupported_profile","no_authoritative_intent","corrupt_profiles_without_intent","legacy_v1_cannot_authorize_b_only"] {
            let root=out.join(variant);copy_tree(&accepted,&root)?;no_locks(&root)?;
            match variant {
                "wrong_payload_same_revision"=>{let path=root.join("active_run.json");let mut wire=active(&path)?;
                    let monotonic=wire.unsigned.monotonic.as_mut().ok_or("old nonempty checkpoint required")?;
                    monotonic.loads_used=monotonic.loads_used.checked_add(1).ok_or("finite changed load count required")?;write_active(&path,wire)?;}
                "unsupported_current"=>{let path=root.join("active_run.json");let mut wire=active(&path)?;wire.unsigned.version=77;write_active(&path,wire)?;}
                "unsupported_profile"=>{fs::write(root.join("player_profile.json"),b"{\"version\":77}")?;}
                "no_authoritative_intent"=>{fs::rename(root.join("application_commit.pending.json"),root.join("unaccepted_intent.tmp"))?;}
                "corrupt_profiles_without_intent"=>{fs::remove_file(root.join("application_commit.pending.json"))?;
                    for name in ["player_profile.json","player_profile.backup.json"]{fs::write(root.join(name),b"{corrupt-profile")?;}}
                "legacy_v1_cannot_authorize_b_only"=>{legacy_v1(&root)?;}
                _=>{},
            }
            denied(&root,&root,if variant=="missing_b"||variant=="corrupt_profiles_without_intent"{&paths.a}else{&paths.b},&out.join(format!("{variant}.json")))?;
        }
        // Even a valid, checksum-correct third configuration is not an old/new
        // member of this accepted transaction. Do not repair it from A's backup.
        let c_save=out.join("valid_c_source");closed_run(&paths.c,&c_save,&out.join("start_c"))?;
        let third=out.join("third_configuration");copy_tree(&accepted,&third)?;no_locks(&third)?;
        fs::copy(c_save.join("active_run.json"),third.join("active_run.json"))?;
        denied(&third,&third,&paths.b,&out.join("third_configuration_preserved.json"))?;
        let damaged=out.join("accepted_corrupt_current");copy_tree(&accepted,&damaged)?;
        fs::write(damaged.join("active_run.json"),b"{corrupt-current")?;
        recover_b(&damaged,&paths.b,&expected,&out.join("accepted_confirmed_old_backup.json"))?;
        let profiles=out.join("accepted_corrupt_profiles");copy_tree(&accepted,&profiles)?;
        for name in ["player_profile.json","player_profile.backup.json"]{fs::write(profiles.join(name),b"{corrupt-profile")?;}
        recover_b(&profiles,&paths.b,&expected,&out.join("accepted_authoritative_profile_replay.json"))?;
        // Exercise both exact old/new copy orders, using bytes published by the
        // actual recovery writer rather than an independent encoder.
        for current_new in [false,true] {
            let label=if current_new{"new_current_old_backup"}else{"old_current_new_backup"};
            let mixed=out.join(label);copy_tree(&accepted,&mixed)?;
            let target=if current_new{"active_run.json"}else{"active_run.backup.json"};
            fs::copy(damaged.join("active_run.json"),mixed.join(target))?;
            recover_b(&mixed,&paths.b,&expected,&out.join(format!("{label}.json")))?;
        }
        accepted_original.ensure_unchanged(&state(&accepted)?)?;
        recover_b(&accepted,&paths.b,&expected,&out.join("accepted_b_only_restart.json"))?;
        for stage in ["profile_renamed","active_renamed","active_backup_synced"] {
            let root=crash(&cases,&out.join(stage),stage,false)?;let expected=expected_new(&root)?;
            let paths:Cases=serde_json::from_slice(&fs::read(out.join(stage).join("cases.json"))?)?;
            recover_b(&root,&paths.b,&expected,&out.join(format!("{stage}_b_only_replay.json")))?;
        }
        let same=crash(&cases,&out.join("same_a_legacy_v1"),"intent_renamed",true)?;
        let paths:Cases=serde_json::from_slice(&fs::read(out.join("same_a_legacy_v1/cases.json"))?)?;
        let legacy_expected=legacy_v1(&same)?;
        // V1 remains readable when every actual original has the exact approved
        // identity. Historical shape grants no new old/new exemption.
        recover_b(&same,&paths.a,&legacy_expected,&out.join("legacy_v1_exact_identity_replayed.json"))?;
        save(&out.join("RESULT.json"),&json!({"passed":true,"actual_crash_stages":5,"actual_crash_processes":6,
            "b_only_replay_requires_authoritative_exact_old_new":true,"preaccept_not_authoritative":true,
            "wrong_payload_and_unsupported_originals_preserved":true,"accepted_profile_replay_preserved":true}))?;Ok(())
    };run().unwrap();
}
