//! Process-separated filesystem integration probe for an exported composed-resource
//! checkpoint. Uses the real Core preparation and durable application adapters;
//! no socket opens. An optional new evidence directory retains all child outputs.
use game_core::{
    application::{ApplicationCommand, CoreApplication},
    game::{
        data::content_pack::{self, ContentSelection},
        encounter_compendium::PlayerProfileState,
    },
};
use game_server::persistence::{
    active_run_store::{ActiveRunSnapshot, ActiveRunStore, StoredCheckpoint},
    player_profile_store::{PlayerProfilePersistence, PlayerProfileStore},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path, process::Command};
use uuid::Uuid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize, Deserialize)]
struct Expected {
    revision: Uuid,
    checkpoint: StoredCheckpoint,
    profile: PlayerProfileState,
    resources: Value,
}
fn read<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn write(path: impl AsRef<Path>, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn open(root: &Path) -> Result<(PlayerProfilePersistence, ActiveRunStore)> {
    let profile =
        PlayerProfilePersistence::bootstrap(PlayerProfileStore::open(root.join("profile"))?);
    let active = ActiveRunStore::open(root.join("active"))?;
    Ok((profile, active))
}
fn restore(
    fixture: &Path,
    checkpoint: &StoredCheckpoint,
    profile: &PlayerProfileState,
) -> Result<CoreApplication> {
    let selection: ContentSelection = read(fixture.join("selection.json"))?;
    let content = content_pack::load(&fixture.join("content"), &selection, &Default::default())?;
    let mut app = CoreApplication::new(content, 0xCA05_1212);
    app.install_authoritative_player_profile(serde_json::to_value(profile)?.into())?;
    app.continue_from_active_run_checkpoint(Uuid::from_u128(1), &checkpoint.to_application())?;
    Ok(app)
}
fn load_command() -> Result<ApplicationCommand> {
    Ok(ApplicationCommand::from_document(
        json!({"type":"load_run_checkpoint"}).into(),
    )?)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceIdentities {
    employee: Uuid,
    equipment_copy: Uuid,
}
fn verify_resource_identities(value: &Value, owners: &ResourceIdentities) -> Result<()> {
    let entries = value["entries"]
        .as_array()
        .ok_or("resource entries absent")?;
    if entries.len() != 2 {
        return Err("expected exactly two resource entries".into());
    }
    for (id, owner) in [
        ("employee_meter", json!({"Employee": owners.employee})),
        (
            "copy_meter",
            json!({"EquipmentCopy": owners.equipment_copy}),
        ),
    ] {
        let matches: Vec<_> = entries
            .iter()
            .filter(|entry| entry["resource"] == id)
            .collect();
        if matches.len() != 1
            || matches[0]["owner"] != owner
            || matches[0]["balance"] != 5
            || matches[0]["revision"] != 2
        {
            return Err(format!("resource identity/value mismatch: {id}").into());
        }
    }
    Ok(())
}
fn resources(fixture: &Path, app: &CoreApplication) -> Result<Value> {
    let value = serde_json::to_value(app.run_resources().ok_or("missing run resources")?)?;
    let identities: ResourceIdentities = read(fixture.join("resource-identities.json"))?;
    verify_resource_identities(&value, &identities)?;
    Ok(value)
}
fn assert_snapshot(actual: &ActiveRunSnapshot, expected: &Expected) {
    assert_eq!(actual.revision, expected.revision);
    assert_eq!(actual.checkpoint.as_ref(), Some(&expected.checkpoint));
}
fn report(phase: &str, value: Value) -> Result<()> {
    println!(
        "{}",
        json!({"phase":phase,"pid":std::process::id(),"result":value})
    );
    std::io::stdout().flush()?;
    Ok(())
}
fn child(phase: &str, fixture: &Path, root: &Path) -> Result<()> {
    if phase == "before-accept" {
        let checkpoint: StoredCheckpoint = read(fixture.join("checkpoint.json"))?;
        let profile: PlayerProfileState = read(fixture.join("profile.json"))?;
        let mut app = restore(fixture, &checkpoint, &profile)?;
        let baseline_resources = resources(fixture, &app)?;
        // Exercise the verifier separately from the store: valid input first,
        // then isolated missing/mixed ownership and definition counterexamples.
        let identities: ResourceIdentities = read(fixture.join("resource-identities.json"))?;
        for fault in ["missing", "employee", "copy", "definition"] {
            let mut bad = baseline_resources.clone();
            if fault == "missing" {
                bad["entries"].as_array_mut().unwrap().pop();
            } else {
                let id = if fault == "copy" {
                    "copy_meter"
                } else {
                    "employee_meter"
                };
                let row = bad["entries"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|row| row["resource"] == id)
                    .unwrap();
                match fault {
                    "employee" => row["owner"] = json!({"Employee": identities.equipment_copy}),
                    "copy" => row["owner"] = json!({"EquipmentCopy": identities.employee}),
                    "definition" => row["resource"] = json!("copy_meter"),
                    _ => unreachable!(),
                }
            }
            assert!(
                verify_resource_identities(&bad, &identities).is_err(),
                "{fault}"
            );
        }
        let entries = app.run_resources().unwrap().entries.as_slice();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|entry| entry.balance == 5));
        let (mut profiles, active) = open(root)?;
        let initial = profiles.accept_application(
            &active,
            active.snapshot()?.revision,
            Some(&checkpoint),
            &profile,
        )?;
        assert!(!initial.recovery_pending);
        let before = active.snapshot()?;
        let expected = Expected {
            revision: before.revision,
            checkpoint,
            profile: profiles.profile().map_err(str::to_owned)?,
            resources: baseline_resources,
        };
        let bytes = fs::read(root.join("active/active_run.json"))?;
        let profile_bytes = fs::read(root.join("profile/player_profile.json"))?;
        let app_before = app.export_active_run_checkpoint()?.unwrap();
        let obstruction = root.join("profile/application_commit.pending.tmp");
        fs::create_dir(&obstruction)?;
        let prepared =
            app.prepare_command(Uuid::from_u128(1), load_command()?, Default::default())?;
        let pending =
            StoredCheckpoint::from(prepared.checkpoint().ok_or("missing prepared checkpoint")?);
        let error = profiles
            .accept_application(
                &active,
                before.revision,
                Some(&pending),
                prepared.player_profile(),
            )
            .expect_err("journal temp obstruction must reject before acceptance");
        drop(prepared);
        assert_eq!(active.snapshot()?, before);
        assert_eq!(fs::read(root.join("active/active_run.json"))?, bytes);
        assert_eq!(
            fs::read(root.join("profile/player_profile.json"))?,
            profile_bytes
        );
        assert!(!root
            .join("profile/application_commit.pending.json")
            .exists());
        assert_eq!(app.export_active_run_checkpoint()?.unwrap(), app_before);
        assert_eq!(resources(fixture, &app)?, expected.resources);
        assert_eq!(profiles.profile().map_err(str::to_owned)?, expected.profile);
        fs::remove_dir(&obstruction)?;
        write(root.join("baseline.expected.json"), &expected)?;
        return report(
            phase,
            json!({"before_accept_failure_preserved":true,"independent_owner_oracle":true,"verifier_counterexamples_rejected":4,"error":error.to_string(),"revision":expected.revision}),
        );
    }

    let baseline: Expected = read(root.join("baseline.expected.json"))?;
    let (mut profiles, active) = open(root)?;
    match phase {
        "accepted-pending" => {
            assert!(!profiles.recover_pending_application(&active)?);
            assert_snapshot(&active.snapshot()?, &baseline);
            let mut app = restore(
                fixture,
                &baseline.checkpoint,
                &profiles.profile().map_err(str::to_owned)?,
            )?;
            assert_eq!(resources(fixture, &app)?, baseline.resources);
            fs::create_dir(root.join("active/active_run.tmp"))?;
            let prepared =
                app.prepare_command(Uuid::from_u128(1), load_command()?, Default::default())?;
            let pending =
                StoredCheckpoint::from(prepared.checkpoint().ok_or("missing prepared checkpoint")?);
            assert_eq!(pending.loads_used, baseline.checkpoint.loads_used + 1);
            let profile = prepared.player_profile().clone();
            let accepted = profiles.accept_application(
                &active,
                baseline.revision,
                Some(&pending),
                &profile,
            )?;
            assert!(accepted.recovery_pending);
            prepared.commit();
            assert_snapshot(&active.snapshot()?, &baseline);
            assert_eq!(resources(fixture, &app)?, baseline.resources);
            let expected = Expected {
                revision: accepted.revision,
                checkpoint: pending,
                profile,
                resources: resources(fixture, &app)?,
            };
            write(root.join("accepted.expected.json"), &expected)?;
            assert!(root
                .join("profile/application_commit.pending.json")
                .is_file());
            assert!(profiles.profile().is_err());
            report(
                phase,
                json!({"durable_acceptance":true,"recovery_pending":true,"accepted_revision":accepted.revision,"loads_used":expected.checkpoint.loads_used}),
            )?;
            // Deliberately bypass destructors: kernel releases locks, while the
            // durable journal and the failed active-file publication remain.
            std::process::exit(86);
        }
        "recovery-blocked" => {
            let journal = fs::read(root.join("profile/application_commit.pending.json"))?;
            let error = profiles
                .recover_pending_application(&active)
                .expect_err("obstruction still present");
            assert_snapshot(&active.snapshot()?, &baseline);
            assert_eq!(
                fs::read(root.join("profile/application_commit.pending.json"))?,
                journal
            );
            assert!(profiles.profile().is_err());
            assert!(profiles
                .accept_application(
                    &active,
                    baseline.revision,
                    Some(&baseline.checkpoint),
                    &baseline.profile
                )
                .is_err());
            assert_eq!(
                fs::read(root.join("profile/application_commit.pending.json"))?,
                journal
            );
            report(
                phase,
                json!({"recovery_failed_preserved":true,"new_acceptance_blocked":true,"error":error.to_string()}),
            )
        }
        "continued" => {
            let continued: Expected = read(root.join("continued.expected.json"))?;
            assert!(!profiles.recover_pending_application(&active)?);
            assert_snapshot(&active.snapshot()?, &continued);
            assert_eq!(
                profiles.profile().map_err(str::to_owned)?,
                continued.profile
            );
            let app = restore(fixture, &continued.checkpoint, &continued.profile)?;
            assert_eq!(resources(fixture, &app)?, baseline.resources);
            assert_eq!(resources(fixture, &app)?, continued.resources);
            assert_eq!(
                app.export_active_run_checkpoint()?.unwrap().loads_used(),
                baseline.checkpoint.loads_used + 2
            );
            let active_bytes = fs::read(root.join("active/active_run.json"))?;
            let profile_bytes = fs::read(root.join("profile/player_profile.json"))?;
            let prior: Expected = read(root.join("accepted.expected.json"))?;
            assert!(profiles
                .accept_application(
                    &active,
                    prior.revision,
                    Some(&prior.checkpoint),
                    &prior.profile
                )
                .is_err());
            assert_eq!(fs::read(root.join("active/active_run.json"))?, active_bytes);
            assert_eq!(
                fs::read(root.join("profile/player_profile.json"))?,
                profile_bytes
            );
            assert!(!profiles.recover_pending_application(&active)?);
            assert!(!root
                .join("profile/application_commit.pending.json")
                .exists());
            report(
                phase,
                json!({"next_command_committed_and_restarted":true,
                "duplicate_recovery_applied":false,"revision":continued.revision,
                "loads_used":continued.checkpoint.loads_used,"resources":continued.resources}),
            )
        }
        "recover" | "verify" => {
            let expected: Expected = read(root.join("accepted.expected.json"))?;
            let applied = profiles.recover_pending_application(&active)?;
            assert_eq!(applied, phase == "recover");
            assert_snapshot(&active.snapshot()?, &expected);
            assert_eq!(profiles.profile().map_err(str::to_owned)?, expected.profile);
            let bytes = fs::read(root.join("active/active_run.json"))?;
            let profile_bytes = fs::read(root.join("profile/player_profile.json"))?;
            assert!(!profiles.recover_pending_application(&active)?);
            assert_eq!(fs::read(root.join("active/active_run.json"))?, bytes);
            assert_eq!(
                fs::read(root.join("profile/player_profile.json"))?,
                profile_bytes
            );
            let mut app = restore(fixture, &expected.checkpoint, &expected.profile)?;
            assert_eq!(resources(fixture, &app)?, expected.resources);
            assert_eq!(expected.resources, baseline.resources);
            assert_eq!(
                app.export_active_run_checkpoint()?.unwrap().loads_used(),
                baseline.checkpoint.loads_used + 1
            );
            assert!(!root
                .join("profile/application_commit.pending.json")
                .exists());
            if phase == "verify" {
                // A fully restored application can prepare the next supported
                // command. Discarding that new candidate must preserve the
                // recovered checkpoint and its once-consumed load count.
                let before_next = app.export_active_run_checkpoint()?.unwrap();
                let next =
                    app.prepare_command(Uuid::from_u128(1), load_command()?, Default::default())?;
                assert_eq!(
                    next.checkpoint().unwrap().loads_used(),
                    expected.checkpoint.loads_used + 1
                );
                drop(next);
                assert_eq!(app.export_active_run_checkpoint()?.unwrap(), before_next);
                assert_eq!(resources(fixture, &app)?, expected.resources);
                // A stale host retry must not publish another revision or
                // overwrite the one accepted candidate after process recovery.
                assert!(profiles
                    .accept_application(
                        &active,
                        baseline.revision,
                        Some(&baseline.checkpoint),
                        &baseline.profile
                    )
                    .is_err());
                assert_snapshot(&active.snapshot()?, &expected);
                assert_eq!(fs::read(root.join("active/active_run.json"))?, bytes);
                assert_eq!(
                    fs::read(root.join("profile/player_profile.json"))?,
                    profile_bytes
                );
                // Continue through the real durable boundary as well as the
                // discarded-candidate path above, then restart once more.
                let next =
                    app.prepare_command(Uuid::from_u128(1), load_command()?, Default::default())?;
                let checkpoint =
                    StoredCheckpoint::from(next.checkpoint().ok_or("missing next checkpoint")?);
                assert_eq!(checkpoint.loads_used, baseline.checkpoint.loads_used + 2);
                let profile = next.player_profile().clone();
                let accepted = profiles.accept_application(
                    &active,
                    expected.revision,
                    Some(&checkpoint),
                    &profile,
                )?;
                assert!(!accepted.recovery_pending);
                next.commit();
                let continued = Expected {
                    revision: accepted.revision,
                    checkpoint,
                    profile,
                    resources: resources(fixture, &app)?,
                };
                assert_snapshot(&active.snapshot()?, &continued);
                assert_eq!(continued.resources, baseline.resources);
                assert_eq!(
                    app.export_active_run_checkpoint()?.unwrap().loads_used(),
                    continued.checkpoint.loads_used
                );
                write(root.join("continued.expected.json"), &continued)?;
                return report(
                    phase,
                    json!({
                        "next_command_prepared_without_mutation":true,
                        "next_command_committed":true,
                        "revision":continued.revision,"loads_used":continued.checkpoint.loads_used,
                        "resources":continued.resources
                    }),
                );
            }
            report(
                phase,
                json!({"recovery_applied":applied,"next_command_prepared_without_mutation":phase == "verify","duplicate_recovery_applied":false,"revision":expected.revision,"loads_used":expected.checkpoint.loads_used,"resources":expected.resources}),
            )
        }
        _ => Err(format!("unknown probe phase: {phase}").into()),
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "--phase") {
        if args.len() != 4 {
            return Err("internal phase requires name, fixture and store root".into());
        }
        return child(
            args[1].to_str().ok_or("phase must be UTF-8")?,
            Path::new(&args[2]),
            Path::new(&args[3]),
        );
    }
    if !(1..=2).contains(&args.len()) {
        return Err(
            "usage: composition_resource_persistence <exported fixture> [new evidence directory]"
                .into(),
        );
    }
    let fixture = Path::new(&args[0]).canonicalize()?;
    let temporary = if args.len() == 1 {
        Some(tempfile::tempdir()?)
    } else {
        None
    };
    let root = if let Some(temp) = &temporary {
        temp.path().to_path_buf()
    } else {
        let path = std::path::PathBuf::from(&args[1]);
        fs::create_dir(&path)?;
        path.canonicalize()?
    };
    let mut reports = Vec::new();
    for phase in [
        "before-accept",
        "accepted-pending",
        "recovery-blocked",
        "recover",
        "verify",
        "continued",
    ] {
        if phase == "recover" {
            fs::remove_dir(root.join("active/active_run.tmp"))?;
        }
        let output = Command::new(std::env::current_exe()?)
            .args([
                std::ffi::OsStr::new("--phase"),
                std::ffi::OsStr::new(phase),
                fixture.as_os_str(),
                root.as_os_str(),
            ])
            .output()?;
        fs::write(root.join(format!("{phase}.stdout.json")), &output.stdout)?;
        fs::write(root.join(format!("{phase}.stderr.log")), &output.stderr)?;
        let expected_exit = if phase == "accepted-pending" { 86 } else { 0 };
        assert_eq!(
            output.status.code(),
            Some(expected_exit),
            "{phase}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut entry: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(entry["phase"], phase);
        entry["exit_code"] = json!(expected_exit);
        reports.push(entry);
    }
    let pids: std::collections::BTreeSet<_> =
        reports.iter().map(|r| r["pid"].as_u64().unwrap()).collect();
    assert_eq!(
        pids.len(),
        reports.len(),
        "each boundary uses a new OS process"
    );
    let result = json!({"status":"passed","processes":reports,"scope":"Linux/host-platform filesystem adapters and new OS processes with composed run resources; no socket, Unity, or CMVA admission proof"});
    write(root.join("result.json"), &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
