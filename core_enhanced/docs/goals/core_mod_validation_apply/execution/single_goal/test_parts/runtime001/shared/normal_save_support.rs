//! Independent normal-host save driver shared by default-release and cfg(test)
//! crash consumers. It only calls the real admission/persistence/Core APIs.
use super::host_server::mod_admission::{AdmissionConfig,AdmissionSession,ApprovedPersistence};
use game_core::{application::{ApplicationCheckpoint,ApplicationCommand,ApplicationSnapshotKind,CoreApplication},
    authoring::CancellationToken,game::{behavior::{GameStateContextDto,RunSnapshotDto},data::content_pack::{ContentSelection,InputLimits}},
    mod_validation::{Candidate,JobState,RequiredConfiguration,TrustedBase},runtime_safety::RuntimeSafetySettings};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::{Path,PathBuf},sync::Arc};
use uuid::Uuid;

#[path="../probe/src/file_state.rs"]
pub mod file_state;
#[path="../probe/src/cache_fixture.rs"]
pub mod cache_fixture;
pub const PLAYER:Uuid=Uuid::from_u128(0xc0de1100000000000000000000001001);
pub type Result<T>=std::result::Result<T,Box<dyn Error>>;
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cases {pub a:PathBuf,pub b:PathBuf,pub c:PathBuf}
pub fn save(path:&Path,value:&impl Serialize)->Result<()> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
pub fn state(path:&Path)->Result<file_state::FileState>{Ok(file_state::FileState::capture(path,65_536,256*1024*1024)?)}
pub fn config(path:&Path)->Result<AdmissionConfig>{AdmissionConfig::read(path).map_err(Into::into)}
pub fn open(config:&AdmissionConfig)->Result<AdmissionSession>{
    let session=AdmissionSession::open(config)?;
    assert_eq!(session.job.state,JobState::Passed,"normal configured issuer must actually accept");
    assert!(session.job.outcome.as_ref().expect("actual normal outcome").is_passed());Ok(session)
}
pub fn acquire(config:&AdmissionConfig)->Result<Candidate>{
    let settings=Arc::new(RuntimeSafetySettings::from_ron("independent normal save",&fs::read(&config.runtime_safety_path)?)?);
    let selection:ContentSelection=serde_json::from_slice(&fs::read(&config.base_selection_path)?)?;
    let base=TrustedBase::acquire(&config.content_root,selection,&InputLimits::default(),settings,&CancellationToken::default())?;
    Ok(Candidate::acquire(&config.content_root,&fs::read(&config.registry_path)?,&base,&InputLimits::default(),&CancellationToken::default())?)
}
pub fn run_snapshot(app:&CoreApplication)->Result<Value>{
    Ok(app.snapshot(ApplicationSnapshotKind::Run).map_err(|e|format!("normal run snapshot: {e:?}"))?.read_model().as_value().clone())
}
pub fn checkpoint(app:&CoreApplication)->Result<Option<ApplicationCheckpoint>>{
    app.export_active_run_checkpoint().map_err(|e|format!("normal checkpoint: {e:?}").into())
}
pub fn execute(session:&AdmissionSession,persistence:&ApprovedPersistence,app:&mut CoreApplication,
    required:&mut RequiredConfiguration,revision:&mut Option<Uuid>,command:Value,force:bool)->Result<Value>{
    let before=checkpoint(app)?;
    let command=ApplicationCommand::from_document(command.into()).map_err(|e|format!("normal command parse: {e:?}"))?;
    let (prepared,next)=session.host.prepare_with_future_generation(app,required,PLAYER,command,Default::default())
        .map_err(|e|format!("normal staged command: {e:?}"))?;
    let (result,pending)=persistence.publish(prepared,&next,revision,before.as_ref(),force)?;
    *required=next;
    if pending{return Err("ordinary positive publication unexpectedly recovery_pending".into());}
    assert_eq!(app.content_identity(),&required.content_identity);
    Ok(serde_json::to_value(result)?)
}
pub fn finish_start(session:&AdmissionSession,persistence:&ApprovedPersistence,app:&mut CoreApplication,
    required:&mut RequiredConfiguration,revision:&mut Option<Uuid>,out:&Path)->Result<()> {
    fs::create_dir(out)?;
    for index in 0..128 {
        let snapshot=run_snapshot(app)?;let run:RunSnapshotDto=serde_json::from_value(snapshot.clone())?;
        let next=if let Some(vn)=run.visual_novel {
            if let Some(choice)=vn.choices.iter().find(|c|c.selectable){
                json!({"type":"select_visual_novel_choice","session_id":vn.session_id,
                    "current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id})
            }else{json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id})}
        }else if let GameStateContextDto::SelectingStarterEmployees{required_count,candidates}=run.game_state_context {
            let ids:Vec<_>=candidates.into_iter().take(required_count).map(|c|c.employee_definition_id).collect();
            if ids.len()!=required_count{return Err("normal starter fixture lacks candidates".into());}
            json!({"type":"select_starter_employees","employee_definition_ids":ids})
        }else{
            assert!(run.roster.combat_eligible_employee_ids.len()>=2);
            assert!(checkpoint(app)?.is_some(),"same-original test requires nonempty continuation");
            save(&out.join("completed.json"),&json!({"required":required,"revision":revision,"run":snapshot}))?;
            return Ok(());
        };
        let result=execute(session,persistence,app,required,revision,next.clone(),true)?;
        save(&out.join(format!("{index:03}.json")),&json!({"command":next,"result":result,"required":required,"revision":revision}))?;
    }
    Err("normal start exceeded finite command bound".into())
}
pub fn create_run(session:&AdmissionSession,persistence:&ApprovedPersistence,out:&Path)
    ->Result<(CoreApplication,RequiredConfiguration,Option<Uuid>)>{
    let (mut app,mut required)=session.host.new_application(77)?;
    app.merge_player_profile(serde_json::to_value(persistence.player_profile()?)?.into()).map_err(|e|format!("approved profile merge: {e:?}"))?;
    let mut revision=None;
    execute(session,persistence,&mut app,&mut required,&mut revision,
        json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false}),true)?;
    finish_start(session,persistence,&mut app,&mut required,&mut revision,out)?;
    Ok((app,required,revision))
}
pub fn continue_original(session:&AdmissionSession,persistence:&ApprovedPersistence,out:&Path)->Result<Value>{
    let stored=persistence.snapshot()?;let original=stored.checkpoint.as_ref().ok_or("nonempty original save required")?;
    let (mut app,required)=session.host.new_application(99)?;
    app.merge_player_profile(serde_json::to_value(persistence.player_profile()?)?.into()).map_err(|e|format!("fresh approved profile merge: {e:?}"))?;
    let result=app.continue_from_active_run_checkpoint(PLAYER,&original.to_application()).map_err(|e|format!("same original Continue: {e:?}"))?;
    assert_eq!(app.content_identity(),&required.content_identity);
    let run=run_snapshot(&app)?;let typed:RunSnapshotDto=serde_json::from_value(run.clone())?;
    assert!(typed.roster.combat_eligible_employee_ids.len()>=2);
    let value=json!({"revision":stored.revision,"required":required,"result":result,"run":run,
        "original_core_sha256":digest(&original.core_payload_json)});
    save(out,&value)?;Ok(value)
}
pub fn digest(bytes:&[u8])->String{use sha2::{Digest,Sha256};format!("{:x}",Sha256::digest(bytes))}
pub fn copy_tree(source:&Path,target:&Path)->Result<()> {
    fs::create_dir(target)?;
    for entry in fs::read_dir(source)? {let entry=entry?;
        if entry.file_type()?.is_dir(){copy_tree(&entry.path(),&target.join(entry.file_name()))?;}
        else if entry.file_type()?.is_file(){fs::copy(entry.path(),target.join(entry.file_name()))?;}
        else{return Err("non-regular save fixture member".into());}}
    Ok(())
}
pub fn no_locks(root:&Path)->Result<()> {
    for name in ["active_run.lock","player_profile.lock"] {let path=root.join(name);if path.exists(){fs::remove_file(path)?;}}
    Ok(())
}
pub fn protected_error(result:std::result::Result<ApprovedPersistence,String>)->Result<String>{
    let error=result.err().ok_or("incompatible original unexpectedly opened")?;
    if !error.to_ascii_lowercase().contains("protect") {return Err(format!("expected protected original, got {error}").into());}
    Ok(error)
}

// Independent supported original-wire encoders used ONLY for corruption and
// compatibility fixtures. Accepted V2 journals themselves always originate at
// a real crash point in the actual writer, never from these helpers.
#[derive(Serialize,Deserialize)]
pub struct Monotonic {pub loads_used:u8,pub run_peak_certified_power:u64}
#[derive(Serialize,Deserialize)]
pub struct ActiveUnsigned {
    pub version:u32,pub revision:Uuid,pub core_payload_base64:Option<String>,pub monotonic:Option<Monotonic>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub required_configuration:Option<RequiredConfiguration>,
}
#[derive(Serialize,Deserialize)]
pub struct ActiveWire {#[serde(flatten)]pub unsigned:ActiveUnsigned,pub checksum_sha256:String}
pub fn write_active(path:&Path,mut wire:ActiveWire)->Result<()> {
    wire.checksum_sha256=digest(&serde_json::to_vec(&wire.unsigned)?);
    fs::write(path,serde_json::to_vec_pretty(&wire)?)?;Ok(())
}
pub fn active(path:&Path)->Result<ActiveWire>{Ok(serde_json::from_slice(&fs::read(path)?)?)}
