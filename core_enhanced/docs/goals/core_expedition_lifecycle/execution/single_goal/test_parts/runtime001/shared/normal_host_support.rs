//! U12 normal admission + publication adapter. The lifecycle assertions are new.
//! API sequencing derives from immutable U11 normal_save_support.rs; no U11
//! fixture, issuer artifact, saved original, or accepted test source is rewritten.
use game_core::{application::{ApplicationCheckpoint, ApplicationCommand, ApplicationCommandKind, ApplicationSnapshotKind, CoreApplication},
    mod_validation::{JobState, RequiredConfiguration}};
use super::host_server::mod_admission::{AdmissionConfig, AdmissionSession, ApprovedPersistence};
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, error::Error, fs, io::Write, path::Path};
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub const PLAYER: Uuid = Uuid::from_u128(0xc0de1200000000000000000000001001);

pub fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.write_all(b"\n")?;
    Ok(())
}

pub fn open(config: &Path) -> Result<AdmissionSession> {
    let session = AdmissionSession::open(&AdmissionConfig::read(config)?)?;
    assert_eq!(session.job.state, JobState::Passed);
    assert!(session.job.outcome.as_ref().expect("normal admission outcome").is_passed());
    Ok(session)
}

pub fn snapshot(app: &CoreApplication, kind: ApplicationSnapshotKind) -> Result<Value> {
    Ok(app.snapshot(kind)?.read_model().as_value().clone())
}

pub fn roots(app: &CoreApplication) -> Result<Value> {
    let snapshots: Vec<_> = ApplicationSnapshotKind::ALL.iter()
        .map(|kind| app.snapshot(*kind)).collect::<std::result::Result<_, _>>()?;
    Ok(json!({"snapshots":snapshots,"continue":app.export_active_run_checkpoint()?
        .map(|checkpoint| checkpoint.core_payload_json().to_vec())}))
}

pub fn new_application(session: &AdmissionSession, persistence: &ApprovedPersistence)
    -> Result<(CoreApplication, RequiredConfiguration, super::host_server::persistence::active_run_store::ActiveRunSnapshot)> {
    let (profile, stored) = persistence.preparation_snapshot()?;
    let (mut app, required) = session.host.new_application(12077)?;
    app.install_authoritative_player_profile(serde_json::to_value(profile)?.into())?;
    Ok((app, required, stored))
}

pub fn restore(session: &AdmissionSession, persistence: &ApprovedPersistence)
    -> Result<(CoreApplication, RequiredConfiguration, Option<Uuid>)> {
    let (mut app, mut required, stored) = new_application(session, persistence)?;
    if let Some(checkpoint) = stored.checkpoint.as_ref() {
        required = stored.required_configuration.clone().ok_or("active save lacks required configuration")?;
        app.continue_from_active_run_checkpoint(PLAYER, &checkpoint.to_application())?;
    }
    Ok((app, required, Some(stored.revision)))
}

pub fn execute(session: &AdmissionSession, persistence: &ApprovedPersistence, app: &mut CoreApplication,
    required: &mut RequiredConfiguration, revision: &mut Option<Uuid>, document: Value) -> Result<Value> {
    execute_with_context(session,persistence,app,required,revision,document,Default::default())
}

pub fn execute_with_context(session: &AdmissionSession, persistence: &ApprovedPersistence, app: &mut CoreApplication,
    required: &mut RequiredConfiguration, revision: &mut Option<Uuid>, document: Value,
    context: game_core::application::ApplicationExecutionContext) -> Result<Value> {
    let before: Option<ApplicationCheckpoint> = app.export_active_run_checkpoint()?;
    let command = ApplicationCommand::from_document(document.into())?;
    let force_revision = command.kind() == ApplicationCommandKind::StartNewGame;
    let (prepared, next) = session.host.prepare_with_future_generation(app, required, PLAYER, command, context)
        .map_err(|error| format!("approved command preparation failed: {error:?}"))?;
    let (result, pending) = persistence.publish(prepared, &next, revision, before.as_ref(), force_revision)?;
    if pending { return Err("positive lifecycle publication recovery_pending".into()); }
    *required = next;
    Ok(serde_json::to_value(result)?)
}

pub fn finish_start(session: &AdmissionSession, persistence: &ApprovedPersistence, app: &mut CoreApplication,
    required: &mut RequiredConfiguration, revision: &mut Option<Uuid>, out: &Path) -> Result<()> {
    for index in 0..128 {
        let run = snapshot(app, ApplicationSnapshotKind::Run)?;
        let command = if !run["visual_novel"].is_null() {
            let vn = &run["visual_novel"];
            if let Some(choice) = vn["choices"].as_array().and_then(|choices| choices.iter().find(|choice| choice["selectable"] == true)) {
                json!({"type":"select_visual_novel_choice","session_id":vn["session_id"],
                    "current_scene_id":vn["current_scene_id"],"choice_id":choice["choice_id"]})
            } else {
                json!({"type":"advance_visual_novel","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"]})
            }
        } else if run["game_state"] == "selecting_starter_employees" {
            let candidates = run["game_state_context"]["candidates"].as_array().ok_or("candidate list")?;
            assert_eq!(candidates.len(), 6);
            assert_eq!(run["game_state_context"]["required_count"], 3);
            for role in ["Dealer", "Tank", "Healer"] {
                assert_eq!(candidates.iter().filter(|candidate| candidate["role"] == role).count(), 2);
            }
            let ids: Vec<_> = candidates.iter().take(3).map(|candidate| candidate["employee_definition_id"].clone()).collect();
            json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]})
        } else {
            assert_eq!(run["game_state"], "viewing_map");
            assert_eq!(run["roster"]["employees"].as_array().ok_or("roster")?.len(), 3);
            assert!(app.export_active_run_checkpoint()?.is_some());
            save(&out.join("started.json"), &roots(app)?)?;
            return Ok(());
        };
        let result = execute(session, persistence, app, required, revision, command.clone())?;
        save(&out.join(format!("start_{index:03}.json")), &json!({"command":command,"result":result,"roots":roots(app)?}))?;
    }
    Err("briefing exceeded finite bound".into())
}

pub fn file_bytes(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
        if !path.exists() { return Ok(()); }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() { visit(root, &entry.path(), out)?; }
            else if entry.file_type()?.is_file() {
                let relative = entry.path().strip_prefix(root)?.to_string_lossy().into_owned();
                if !relative.ends_with(".lock") { out.insert(relative, fs::read(entry.path())?); }
            } else { return Err("non-regular persistence fixture".into()); }
        }
        Ok(())
    }
    let mut out = BTreeMap::new(); visit(root, root, &mut out)?; Ok(out)
}
