//! Normal local launcher. All gameplay consumes an issuer-approved host and the
//! same persistence adapter as the server. Static authoring tools are diagnostic.
use game_core::application::{
    ApplicationCommand, ApplicationCommandKind, ApplicationDocument, ApplicationExecutionContext,
    ApplicationSnapshotKind,
};
use game_server::mod_admission::{AdmissionConfig, AdmissionSession, ApprovedPersistence};
use std::{path::Path, sync::Arc};
#[cfg(any(feature = "runtime-measurement", feature = "cmva-lab-measurement"))]
#[global_allocator]
static ALLOCATOR: game_server::runtime_measurement::CountingAllocator<'static, std::alloc::System> =
    game_server::runtime_measurement::process_allocator();
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match game_server::mod_admission::worker::entry(&args) {
        Ok(true) => return,
        Ok(false) => normal(&args),
        Err(e) => Err(e.to_string()),
    };
    match result {
        Ok(value) => println!("{}", serde_json::to_string(&value).expect("JSON result")),
        Err(reason) => {
            eprintln!(
                "{}",
                serde_json::json!({"status":"refused", "reason":reason})
            );
            std::process::exit(2);
        }
    }
}
fn normal(args: &[String]) -> Result<serde_json::Value, String> {
    if let Some(value) = game_server::mod_admission::administration(args)? {
        return Ok(value);
    }
    if args.len() < 2 {
        return Err("usage: mod_launcher <validate|query|apply|continue|run> <admission.json> [persistence-root] [commands.json]".into());
    }
    let config = AdmissionConfig::read(Path::new(&args[1]))?;
    if args[0] == "validate" {
        if args.len() != 2 {
            return Err("validate takes an admission configuration".into());
        }
        return serde_json::to_value(AdmissionSession::validate(&config)?)
            .map_err(|e| e.to_string());
    }
    let session = AdmissionSession::open(&config)?;
    if matches!(args[0].as_str(), "query" | "apply") {
        if args.len() != 2 {
            return Err("query/apply takes an admission configuration".into());
        }
        return Ok(serde_json::json!({"selection":session.query()?, "job":session.job}));
    }
    if !matches!(args[0].as_str(), "continue" | "run")
        || args.len() != if args[0] == "run" { 4 } else { 3 }
    {
        return Err(
            "continue/run requires an explicit persistence root and run requires commands.json"
                .into(),
        );
    }
    let persistence = ApprovedPersistence::open(Path::new(&args[2]), Arc::clone(&session.host))?;
    let player = uuid::Uuid::from_u128(1);
    let (mut application, mut required) =
        session.host.new_application(1).map_err(|e| e.to_string())?;
    let (profile, snapshot) = persistence.preparation_snapshot()?;
    application.install_authoritative_player_profile(ApplicationDocument::new(
        serde_json::to_value(profile).map_err(|e| e.to_string())?,
    )).map_err(|e| e.to_string())?;
    let mut revision = Some(snapshot.revision);
    let mut results = Vec::new();
    if args[0] == "continue" {
        let checkpoint = snapshot
            .checkpoint
            .ok_or_else(|| "no active run to continue".to_owned())?;
        let result = application
            .continue_from_active_run_checkpoint(player, &checkpoint.to_application())
            .map_err(|e| e.to_string())?;
        results.push(serde_json::to_value(result).map_err(|e| e.to_string())?);
    } else {
        let metadata = std::fs::metadata(&args[3]).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
            return Err("finite regular command script required".into());
        }
        let commands: Vec<serde_json::Value> =
            serde_json::from_slice(&std::fs::read(&args[3]).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if commands.len() > 256 {
            return Err("finite command script ceiling".into());
        }
        for value in commands {
            let command = ApplicationCommand::from_document(ApplicationDocument::new(value))
                .map_err(|e| e.to_string())?;
            if command.kind() == ApplicationCommandKind::ContinueRun {
                let snapshot = persistence.snapshot()?;
                let checkpoint = snapshot
                    .checkpoint
                    .ok_or_else(|| "no active run to continue".to_owned())?;
                let result = application
                    .continue_from_active_run_checkpoint(player, &checkpoint.to_application())
                    .map_err(|e| e.to_string())?;
                revision = Some(snapshot.revision);
                required = snapshot.required_configuration.unwrap_or(required);
                results.push(serde_json::to_value(result).map_err(|e| e.to_string())?);
                continue;
            }
            let start = command.kind() == ApplicationCommandKind::StartNewGame;
            let before = application
                .export_active_run_checkpoint()
                .map_err(|e| e.to_string())?;
            let (prepared, next_required) = session
                .host
                .prepare_with_future_generation(
                    &mut application,
                    &required,
                    player,
                    command,
                    ApplicationExecutionContext::default(),
                )
                .map_err(|e| format!("{e:?}"))?;
            let (result, recovery_pending) = persistence.publish(
                prepared,
                &next_required,
                &mut revision,
                before.as_ref(),
                start,
            )?;
            required = next_required;
            results
                .push(serde_json::json!({"result": result, "recovery_pending": recovery_pending}));
            if recovery_pending {
                return Ok(
                    serde_json::json!({"status":"recovery_pending", "selection":session.query()?, "results":results}),
                );
            }
        }
    }
    Ok(
        serde_json::json!({"status":"accepted", "selection":session.query()?, "required_configuration":required, "results":results, "player_profile":application.snapshot(ApplicationSnapshotKind::PlayerProfile).map_err(|e| e.to_string())?, "continue":persistence.snapshot()?.checkpoint.is_some()}),
    )
}
